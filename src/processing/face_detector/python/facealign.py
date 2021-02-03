import face_alignment

fa = 0  # placeholder


def gen_face_detector_global(dimension, dev='cpu', flip=False, fd='sfd',
                             fd_kw={"filter_threshold": 0.8}):
    global fa
    d = face_alignment.LandmarksType._3D
    if dimension == "2D":
        d = face_alignment.LandmarksType._2D
    elif dimension == "2.5D":
        d = face_alignment.LandmarksType._2D
    fa = face_alignment.FaceAlignment(d, device=dev, flip_input=flip, face_detector=fd, face_detector_kwargs=fd_kw)


def gen_face_detector_local(dimension, dev='cpu', flip=False, fd='sfd',
                            fd_kw={"filter_threshold": 0.8}):
    d = face_alignment.LandmarksType._3D
    if dimension == "2D":
        d = face_alignment.LandmarksType._2D
    elif dimension == "2.5D":
        d = face_alignment.LandmarksType._2D
    return face_alignment.FaceAlignment(d, device=dev, flip_input=flip, face_detector=fd, face_detector_kwargs=fd_kw)


def get_current():
    return fa


def detect_faces():
    global fa
    # return fa.detect_
