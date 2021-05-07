//     Open2DHolo - Open 2D Holo, a program to procedurally animate your face onto an 3D Model.
//     Copyright (C) 2020-2021 l1npengtul
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU General Public License as published by
//     the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//     GNU General Public License for more details.
//
//     You should have received a copy of the GNU General Public License
//     along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{error::model_error::{self, ModelError}, util::camera::device_utils::{DeviceConfig, PossibleDevice, Resolution}};
use facial_processing::utils::misc::{BackendProviders, EulerAngles, Point2D};
use gdnative::{TRef, api::Skeleton, core_types::Transform};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, ops::Deref};
use k::{Isometry3, Node, NodeBuilder, Translation3, UnitQuaternion};
use nalgebra::{Quaternion, Translation, Vector4};

// TODO: Change to acutal data format
#[derive(Clone)]
pub enum MessageType {
    Die(u8),
    SetDevice {
        name: Option<String>,
        device: PossibleDevice,
    },
    ChangeDevice(DeviceConfig),
}

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    Dlib,
}

#[derive(Clone, Copy, Debug)]
pub struct BackendConfig {
    backend: Backend,
    input_src_original: Resolution,
    input_src_scaled: Resolution,
    // DeviceConfig TODO: wait for TVM
}
impl BackendConfig {
    pub fn new(res: Resolution, backend: Backend) -> Self {
        // let mut scaled_res = Resolution::new(640, 480);
        // let mut scale_x = 1_f64;
        // let mut scale_y = 1_f64;
        // if res.x < 640 {

        // }
        BackendConfig {
            backend,
            input_src_original: res,
            input_src_scaled: res, // TODO: find min scale factor
        }
    }

    pub fn backend_as_facial(&self) -> BackendProviders {
        match self.backend {
            Backend::Dlib => {
                BackendProviders::DLib {
                    face_alignment_path: globalize_path!("res://models/facial-processing-rs-models/shape_predictor_68_face_landmarks.dat")
                }
            }
        }
    }

    pub fn res(&self) -> Resolution {
        self.input_src_original
    }
}

#[derive(Clone)]
pub struct FullyCalculatedPacket {
    // pub landmarks: FaceLandmark,
    pub landmarks: Vec<Point2D>,
    pub euler: EulerAngles,
    // pub eye_positions: [Eye; 2],
}

// TODO: Add serde serialize/deserialize to RON or equivalent

fn make_allow_bool(s: String) -> bool {
    matches!(s.as_str(), "Allow")
}

fn make_option_str(s: String) -> Option<String> {
    let made_opt = s.as_str();
    if made_opt.is_empty() {
        None
    } else {
        Some(made_opt.to_string())
    }
}

// VRoid JSON decoder
#[derive(Serialize, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct VrmPermBuilder {
    pub version: String,
    pub author: String,
    pub contactInformation: String,
    pub reference: String,
    pub title: String,
    pub texture: i32,
    pub allowedUserName: String,
    pub violentUssageName: String,
    pub sexualUssageName: String,
    pub commercialUssageName: String,
    pub otherPermissionUrl: String,
    pub licenseName: String,
    pub otherLicenseUrl: String,
}
impl VrmPermBuilder {
    pub fn new(vrm_path: String) -> Self {
        let vrm_file = File::open(vrm_path).unwrap();
        let vrm = gltf::Glb::from_reader(vrm_file).unwrap();
        let json = vrm.json;
        let s = match std::str::from_utf8(json.as_ref()) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        let json_parsed = json::parse(s).unwrap();
        let json_iter = json_parsed.entries();
        let mut extensions_vec: Vec<String> = vec![];
        let mut json_string: String = String::new();
        for (tag, value) in json_iter {
            if tag == "extensionsUsed" {
                for mem in value.members() {
                    extensions_vec.push(String::from(mem.as_str().unwrap()))
                }
            } else if tag == "extensions" {
                for (k, v) in value.entries() {
                    if extensions_vec.contains(&String::from(k)) {
                        for (a, b) in v.entries() {
                            if a == "meta" {
                                json_string = b.to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }

        serde_json::from_str(&json_string).unwrap()
    }

    pub fn split(self) -> (VRMStylePermissions, CreatorMetadata) {
        let violence = make_allow_bool(self.violentUssageName);
        let sexual = make_allow_bool(self.sexualUssageName);
        let commer = make_allow_bool(self.commercialUssageName);
        let permurl = make_option_str(self.otherPermissionUrl);
        let license = {
            match (self.licenseName.as_str(), self.otherLicenseUrl.as_str()) {
                ("", y) if !y.is_empty() => y.to_string(),
                (x, "") if !x.is_empty() => x.to_string(),
                (x, y) if !x.is_empty() && y.is_empty() => {
                    format!("{} / {}", x, y)
                }
                (_, _) => "All Rights Reserved".to_string(),
            }
        };
        let vrmstyle = VRMStylePermissions::new(
            AllowedPersons::from(self.allowedUserName),
            violence,
            sexual,
            commer,
            permurl,
            license,
        );

        let name = make_option_str(self.title);
        let author = make_option_str(self.author);
        let contact = make_option_str(self.contactInformation);
        let reference = make_option_str(self.reference);
        let version = make_option_str(self.version);
        let creatormeta = CreatorMetadata::new(name, author, contact, reference, version);

        (vrmstyle, creatormeta)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllowedPersons {
    Everyone,
    ExplicitlyLicensedPerson,
    OnlyAuthor,
}
impl From<String> for AllowedPersons {
    fn from(f: String) -> Self {
        match f.as_ref() {
            "ExplicitlyLicensedPerson" => AllowedPersons::ExplicitlyLicensedPerson,
            "OnlyAuthor" => AllowedPersons::OnlyAuthor,
            _ => AllowedPersons::Everyone,
        }
    }
}
/// These are permissions taken from `VRoid`
/// You can check for futher details on Pixiv's VROID FAQ
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VRMStylePermissions {
    allowed_persons: AllowedPersons,
    allow_violence: bool,
    allow_sexual: bool,
    commercial: bool,
    additional_url: Option<String>,
    license: String,
}
impl VRMStylePermissions {
    pub fn new(
        allowed_persons: AllowedPersons,
        allow_violence: bool,
        allow_sexual: bool,
        commercial: bool,
        additional_url: Option<String>,
        license: String,
    ) -> Self {
        VRMStylePermissions {
            allowed_persons,
            allow_violence,
            allow_sexual,
            commercial,
            additional_url,
            license,
        }
    }

    /// check the VRM model's allowed persons.
    pub fn allowed_persons(&self) -> AllowedPersons {
        self.allowed_persons
    }

    /// Check if the VRM model allows violence.
    pub fn allow_violence(&self) -> bool {
        self.allow_violence
    }

    /// Check if the VRM model allows sexual.
    pub fn allow_sexual(&self) -> bool {
        self.allow_sexual
    }

    /// Check if the VRM model allows commercial use.
    pub fn commercial(&self) -> bool {
        self.commercial
    }

    /// Check if the VRM model's additional permission url.
    pub fn additional_url(&self) -> &Option<String> {
        &self.additional_url
    }

    /// Get a reference to the v r m style permissions's license.
    pub fn license(&self) -> &String {
        &self.license
    }
}
impl Default for VRMStylePermissions {
    fn default() -> Self {
        VRMStylePermissions {
            allowed_persons: AllowedPersons::OnlyAuthor,
            allow_violence: false,
            allow_sexual: false,
            commercial: false,
            additional_url: None,
            license: "All Rights Reserved".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatorMetadata {
    name: Option<String>,
    author: Option<String>,
    contact: Option<String>,
    reference: Option<String>,
    version: Option<String>,
}
impl CreatorMetadata {
    pub fn new(
        name: Option<String>,
        author: Option<String>,
        contact: Option<String>,
        reference: Option<String>,
        version: Option<String>,
    ) -> Self {
        CreatorMetadata {
            name,
            author,
            contact,
            reference,
            version,
        }
    }

    /// Get a reference to the creator's name.
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    /// Get a reference to the creator's author.
    pub fn author(&self) -> &Option<String> {
        &self.author
    }

    /// Get a reference to the creator's contact.
    pub fn contact(&self) -> &Option<String> {
        &self.contact
    }

    /// Get a reference to the creator's reference.
    pub fn reference(&self) -> &Option<String> {
        &self.reference
    }

    /// Get a reference to the creator's version.
    pub fn version(&self) -> &Option<String> {
        &self.version
    }
}

pub struct MdlRefBuilder {
    display_name: String,
    model_path: String,
    tscn_path: String, // TODO: Deprecate in 4.0
    creator_meta: Option<CreatorMetadata>,
    vrm_style_perms: Option<VRMStylePermissions>,
}
impl MdlRefBuilder {
    pub fn new() -> Self {
        MdlRefBuilder::default()
    }

    pub fn with_display_name(mut self, name: String) -> MdlRefBuilder {
        self.display_name = name;
        self
    }

    pub fn check_empty_displayname(&self) -> bool {
        if self.display_name == *"" {
            return true;
        }
        false
    }

    pub fn with_model_path(mut self, path: String) -> MdlRefBuilder {
        self.model_path = path;
        self
    }

    pub fn with_tscn_path(mut self, tscn_path: String) -> MdlRefBuilder {
        self.tscn_path = tscn_path;
        self
    }

    // pub fn with_license_by_name(mut self, license: String) -> MdlRefBuilder {
    //     self.license = Some(license);
    //     self
    // }

    // pub fn with_spdx_license(mut self, license: &dyn License) -> MdlRefBuilder {
    //     self.license = Some(license.name().to_string());
    //     self
    // }

    pub fn from_vrm_meta_json(path: String) -> MdlRefBuilder {
        let vrm: VrmPermBuilder = VrmPermBuilder::new(path);
        let (vrmstyle, creatormeta) = vrm.split();
        let mdlref = MdlRefBuilder {
            display_name: creatormeta.name().clone().unwrap_or_else(|| "".to_string()),
            creator_meta: Some(creatormeta),
            vrm_style_perms: Some(vrmstyle),
            ..MdlRefBuilder::default()
        };
        mdlref
    }

    pub fn build(self) -> ModelReference {
        ModelReference {
            display_name: self.display_name,
            model_path: self.model_path,
            tscn_path: self.tscn_path,
            creator_meta: self.creator_meta,
            vrm_style_perms: self.vrm_style_perms,
        }
    }
}

impl Default for MdlRefBuilder {
    fn default() -> Self {
        MdlRefBuilder {
            display_name: "".to_string(),
            model_path: "".to_string(),
            tscn_path: "".to_string(),
            creator_meta: None,
            vrm_style_perms: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelReference {
    display_name: String,
    model_path: String,
    tscn_path: String, // TODO: Deprecate in 4.0
    creator_meta: Option<CreatorMetadata>,
    vrm_style_perms: Option<VRMStylePermissions>,
}
impl ModelReference {
    /// Get a reference to the model reference's display name.
    pub fn display_name(&self) -> &String {
        &self.display_name
    }

    /// the model reference's model path.
    pub fn model_path(&self) -> &String {
        &self.model_path
    }

    /// the model reference's model path, global path not res://
    pub fn globalized_model_path(&self) -> String {
        globalize_path!(self.model_path())
    }

    /// the model reference's tscn path.
    pub fn tscn_path(&self) -> &String {
        &self.tscn_path
    }

    /// the model reference's Scene path, global path not res://
    pub fn globalized_tscn_path(&self) -> String {
        globalize_path!(self.tscn_path())
    }

    /// Get a reference to the model reference's creator meta.
    pub fn creator_meta(&self) -> &Option<CreatorMetadata> {
        &self.creator_meta
    }

    /// Get a reference to the model reference's vrm style perms.
    pub fn vrm_style_perms(&self) -> &Option<VRMStylePermissions> {
        &self.vrm_style_perms
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ArbitaryVecRead {
    held_data: Vec<u8>,
}

impl ArbitaryVecRead {
    pub fn new(data: Vec<u8>) -> Self {
        ArbitaryVecRead { held_data: data }
    }

    pub fn replace(&mut self, new_data: Vec<u8>) -> Vec<u8> {
        std::mem::replace(&mut self.held_data, new_data)
    }
}

#[allow(clippy::needless_range_loop)]
impl Read for ArbitaryVecRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut iter = self.held_data.iter();
        for i in 0..buf.len() {
            if let Some(byte) = iter.next() {
                buf[i] = *(byte as &u8);
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut iter = self.held_data.iter();
        for i in 0..buf.len() {
            if let Some(byte) = iter.next() {
                buf.push(*(byte as &u8));
            } else {
                return Ok(i);
            }
        }
        Ok(buf.len())
    }
}

// FIXME: WARNING! DO TEST IF X,Y,Z in Godot matches up to k X,Y,Z ?
pub struct IkChain {
    skelly_id_map: HashMap<i64, usize>,
}

impl IkChain {
    pub fn new(skeleton: TRef<Skeleton>) -> Result<Self, ModelError> {
        if skeleton.get_bone_count() >= 1 {
            return Err(
                ModelError::InvalidBoneNumberError(skeleton.get_bone_count(), "".to_string())
            );
        }
        skeleton.localize_rests();
        let node_map_string = HashMap::new();

        for bone_id in 0..skeleton.get_bone_count() {
            let bone_name = skeleton.get_bone_name(bone_id).to_string();
            let bone_rest = transform_into_isometry3(skeleton.get_bone_rest(bone_id));

            let mut node = NodeBuilder::new()
            .name(bone_name.as_str())
            .origin(bone_rest)
            .joint_type()
        }

        Err(ModelError::InvalidBoneNumberError(0, "".to_string()))
    }
}

// FIXME: WARNING! DO TEST IF X,Y,Z in Godot matches up to k X,Y,Z ? 
#[inline]
fn transform_into_isometry3(transform: Transform) -> Isometry3<f32> {
    let position = transform.origin; // X, Y, Z
    let rotation = transform.basis.to_quat().normalize(); // X, Y, Z

    let position_translation: Translation3<f32> = Translation3::new(position.x, position.y, position.z);
    let rotation_angle = UnitQuaternion::from_quaternion(Quaternion::new(rotation.i, rotation.j, rotation.k, rotation.r));
    Isometry3::from_parts(position_translation, rotation_angle)
}

// Converts Z-Axis up centric to Y Axis up centric coordinates. 
// TODO
fn convert_xyz_y_up(iso: Isometry3<f32>) /*-> Isometry3<f32> */ {

}
