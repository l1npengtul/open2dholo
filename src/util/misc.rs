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

use facial_processing::utils::{
    eyes::Eye,
    face::FaceLandmark,
    misc::{BackendProviders, EulerAngles},
};
use license::License;
use serde::{Deserialize, Serialize};
use crate::util::camera::device_utils::{DeviceConfig, PossibleDevice, Resolution};

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
    pub landmarks: FaceLandmark,
    pub euler: EulerAngles,
    pub eye_positions: [Eye; 2],
}


// TODO: Add serde serialize/deserialize to RON or equivalent

// VRoid JSON decoder
#[derive(Serialize, Debug, Deserialize)]
struct VrmPermBuilder {
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
    pub otherLicenseUrl: String
}
impl VrmPermBuilder {
    pub fn split(self) -> (VRMStylePermissions, CreatorMetadata) {
        // TODO
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
}
impl VRMStylePermissions {
    /// check the VRM model's allowed persons.
    pub fn allowed_persons(&self) -> &AllowedPersons {
        &self.allowed_persons
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
}
impl Default for VRMStylePermissions {
    fn default() -> Self {
        VRMStylePermissions {
            allowed_persons: AllowedPersons::AuthorOnly,
            allow_violence: false,
            allow_sexual: false,
            commercial: false,
            additional_url: None,

        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatorMetadata {
    name: Option<String>,
    author: Option<String>,
    contact: Option<String>,
    reference: Option<String>,
    version: Option<String>
}
impl CreatorMetadata {


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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllowedPersons {
    AuthorOnly,
    PermittedOnly(String),
    Anyone,
}

pub struct MdlRefBuilder {
    display_name: String,
    model_path: String,
    tscn_path: String, // TODO: Deprecate in 4.0
    license: Option<String>,
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

    pub fn with_model_path(mut self, path: String) -> MdlRefBuilder {
        self.model_path = path;
        self
    }

    pub fn with_tscn_path(mut self, tscn_path: String) -> MdlRefBuilder {
        self.tscn_path = tscn_path;
        self
    }

    pub fn with_license_by_name(mut self, license: String) -> MdlRefBuilder {
        self.license = Some(license);
        self
    }

    pub fn with_spdx_license(mut self, license: &dyn License) -> MdlRefBuilder {
        self.license = Some(license.name().to_string());
        self
    }

    pub fn with_vrm_meta_json(mut self, json: String) -> MdlRefBuilder {
        let json_parsed = json::parse(json).unwrap();
        let json_iter = json_parsed.entries();
        let mut extensions_vec: Vec<String> = vec![];
        let mut json_string: String = String::new();
        for (tag, value) in json_iter {
            if tag == "extensionsUsed" {
                for mem in value.members() {
                    extensions_vec.push(String::from(mem.as_str().unwrap()))
                }
            }
            if tag == "extensions" {
                for (k,v) in value.entries() {
                    if extensions_vec.contains(&String::from(k)) {
                        for (a,b) in v.entries() {
                            if a == "meta" {
                                println!("{}",b);
                                json_string = b.to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }
    
        let vrm_perm: VrmPermBuilder = serde_json::from_str(&json_string).unwrap();


    }
}

impl Default for MdlRefBuilder {
    fn default() -> Self {
        MdlRefBuilder {
            display_name: "".to_string(),
            model_path: "".to_string(),
            tscn_path: "".to_string(),
            license: None,
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
    license: Option<String>,
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

    /// get the model reference's license.
    pub fn license(&self) -> &Option<String> {
        &self.license
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