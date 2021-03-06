#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_archive;

#[derive(Debug, Serialize)]
struct Item {
    pub name: String,
    pub sub: SubItem,
    pub cert: String,
}

#[derive(Debug, Serialize)]
struct SubItem {
    pub id: i32,
}

#[cfg(feature = "enable-zip")]
#[test]
fn serialize_struct_zip() {

    use serde_archive::ser::zip_writer;
    let item = Item {
        name: String::from("test"),
        sub: SubItem { id: 12 },
        cert: "-----BEGIN CERTIFICATE-----
MIIFADCCA+igAwIBAgIJAMHa46+zz1PiMA0GCSqGSIb3DQEBCwUAMIGwMQswCQYD
VQQGEwJGUjELMAkGA1UECBMCRlIxGTAXBgNVBAcTEFNhaW50SmVhbkRlQnJheWUx
ETAPBgNVBAoTCFJlc29TYWZlMRUwEwYDVQQLEwxFeHBsb2l0YXRpb24xFDASBgNV
BAMTC1Jlc29TYWZlIENBMRAwDgYDVQQpEwdFYXN5UlNBMScwJQYJKoZIhvcNAQkB
FhhleHBsb2l0YXRpb25AcmVzb3NhZmUuZnIwHhcNMTgwODIwMDgwMTQyWhcNMjgw
ODE3MDgwMTQyWjCBsDELMAkGA1UEBhMCRlIxCzAJBgNVBAgTAkZSMRkwFwYDVQQH
ExBTYWludEplYW5EZUJyYXllMREwDwYDVQQKEwhSZXNvU2FmZTEVMBMGA1UECxMM
RXhwbG9pdGF0aW9uMRQwEgYDVQQDEwtSZXNvU2FmZSBDQTEQMA4GA1UEKRMHRWFz
eVJTQTEnMCUGCSqGSIb3DQEJARYYZXhwbG9pdGF0aW9uQHJlc29zYWZlLmZyMIIB
IjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA+7fQerba+cpSo6pdrUz0b2bP
dNGy1610HgN9yE24fzLR3oHyKT6Q6j0kLWh//vNgcpuZRLvWxPBnWBBlcALiPy8N
eKgRBfkf7i4cEC8noDIyRHqc6cKb5juvS9kCJnAp9TDZNt3+Pi6iK2uIYDQdJEgW
mbEhM0k3clLC5I/zzKZEPuFksCdvG+BjahU+hyglnH8KSJNMnVAnC2OBJ+gEq51o
Eyub7Y5/d4oOHtaZUCoGYUTgO1DHfNrjLt5tzBZt2Hfph3kZtU5f6W/YQdNVCM+J
jamlqEuEsF+0RxLBEtZzGZG9FJKj2R/+9psvvNW4rypxz/vbN7yibjy8BLFe1wID
AQABo4IBGTCCARUwHQYDVR0OBBYEFNpFfCIA5xPzMYM8QR7HvkDo6ZXhMIHlBgNV
HSMEgd0wgdqAFNpFfCIA5xPzMYM8QR7HvkDo6ZXhoYG2pIGzMIGwMQswCQYDVQQG
EwJGUjELMAkGA1UECBMCRlIxGTAXBgNVBAcTEFNhaW50SmVhbkRlQnJheWUxETAP
BgNVBAoTCFJlc29TYWZlMRUwEwYDVQQLEwxFeHBsb2l0YXRpb24xFDASBgNVBAMT
C1Jlc29TYWZlIENBMRAwDgYDVQQpEwdFYXN5UlNBMScwJQYJKoZIhvcNAQkBFhhl
eHBsb2l0YXRpb25AcmVzb3NhZmUuZnKCCQDB2uOvs89T4jAMBgNVHRMEBTADAQH/
MA0GCSqGSIb3DQEBCwUAA4IBAQARTxvitvA4/95YnC6HcRhfIB720FRf6Tpp7Shy
22g0jjCbJZsk2OEJy1Vqr530gYoNmmYIoaJvC1jNt2l4A65DuJM8xThaCcVsNY+O
wfZj7VHz6m60FIooEgDaHOtxUF9YN2g+5tSArH5RFgD6X6noYHKjDK4GCpUnj6oN
Tzea+UKbNR8eGnQKNUeZHYmCYlayZewBYFwwt1XFyaaqjFHO1ggOe81ytF8gVX9Z
25iOaqupLe3/q1z63nXhfA+nQ4H1//YFDrKRw4fiRoedEJDUfOzbXkOxVCeU/z4l
ERHpT66G1wzSMm+ggi7yT4BJaz9slMLzZxyMKEPYNldJ1AJk
-----END CERTIFICATE-----"
            .to_string(),
    };

    let file = std::fs::File::create("/tmp/serde_zip-test.zip").unwrap();
    let writer = zip_writer::ZipWriter::new(file, zip_writer::ZipCompression::Bzip2);
    assert_eq!(serde_archive::to_writer(writer, &item), Ok(()));
}

#[cfg(feature = "enable-tar")]
#[test]
fn serialize_struct_tar() {

    let item = Item {
        name: String::from("test"),
        sub: SubItem { id: 12 },
        cert: "-----BEGIN CERTIFICATE-----
MIIFADCCA+igAwIBAgIJAMHa46+zz1PiMA0GCSqGSIb3DQEBCwUAMIGwMQswCQYD
VQQGEwJGUjELMAkGA1UECBMCRlIxGTAXBgNVBAcTEFNhaW50SmVhbkRlQnJheWUx
ETAPBgNVBAoTCFJlc29TYWZlMRUwEwYDVQQLEwxFeHBsb2l0YXRpb24xFDASBgNV
BAMTC1Jlc29TYWZlIENBMRAwDgYDVQQpEwdFYXN5UlNBMScwJQYJKoZIhvcNAQkB
FhhleHBsb2l0YXRpb25AcmVzb3NhZmUuZnIwHhcNMTgwODIwMDgwMTQyWhcNMjgw
ODE3MDgwMTQyWjCBsDELMAkGA1UEBhMCRlIxCzAJBgNVBAgTAkZSMRkwFwYDVQQH
ExBTYWludEplYW5EZUJyYXllMREwDwYDVQQKEwhSZXNvU2FmZTEVMBMGA1UECxMM
RXhwbG9pdGF0aW9uMRQwEgYDVQQDEwtSZXNvU2FmZSBDQTEQMA4GA1UEKRMHRWFz
eVJTQTEnMCUGCSqGSIb3DQEJARYYZXhwbG9pdGF0aW9uQHJlc29zYWZlLmZyMIIB
IjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA+7fQerba+cpSo6pdrUz0b2bP
dNGy1610HgN9yE24fzLR3oHyKT6Q6j0kLWh//vNgcpuZRLvWxPBnWBBlcALiPy8N
eKgRBfkf7i4cEC8noDIyRHqc6cKb5juvS9kCJnAp9TDZNt3+Pi6iK2uIYDQdJEgW
mbEhM0k3clLC5I/zzKZEPuFksCdvG+BjahU+hyglnH8KSJNMnVAnC2OBJ+gEq51o
Eyub7Y5/d4oOHtaZUCoGYUTgO1DHfNrjLt5tzBZt2Hfph3kZtU5f6W/YQdNVCM+J
jamlqEuEsF+0RxLBEtZzGZG9FJKj2R/+9psvvNW4rypxz/vbN7yibjy8BLFe1wID
AQABo4IBGTCCARUwHQYDVR0OBBYEFNpFfCIA5xPzMYM8QR7HvkDo6ZXhMIHlBgNV
HSMEgd0wgdqAFNpFfCIA5xPzMYM8QR7HvkDo6ZXhoYG2pIGzMIGwMQswCQYDVQQG
EwJGUjELMAkGA1UECBMCRlIxGTAXBgNVBAcTEFNhaW50SmVhbkRlQnJheWUxETAP
BgNVBAoTCFJlc29TYWZlMRUwEwYDVQQLEwxFeHBsb2l0YXRpb24xFDASBgNVBAMT
C1Jlc29TYWZlIENBMRAwDgYDVQQpEwdFYXN5UlNBMScwJQYJKoZIhvcNAQkBFhhl
eHBsb2l0YXRpb25AcmVzb3NhZmUuZnKCCQDB2uOvs89T4jAMBgNVHRMEBTADAQH/
MA0GCSqGSIb3DQEBCwUAA4IBAQARTxvitvA4/95YnC6HcRhfIB720FRf6Tpp7Shy
22g0jjCbJZsk2OEJy1Vqr530gYoNmmYIoaJvC1jNt2l4A65DuJM8xThaCcVsNY+O
wfZj7VHz6m60FIooEgDaHOtxUF9YN2g+5tSArH5RFgD6X6noYHKjDK4GCpUnj6oN
Tzea+UKbNR8eGnQKNUeZHYmCYlayZewBYFwwt1XFyaaqjFHO1ggOe81ytF8gVX9Z
25iOaqupLe3/q1z63nXhfA+nQ4H1//YFDrKRw4fiRoedEJDUfOzbXkOxVCeU/z4l
ERHpT66G1wzSMm+ggi7yT4BJaz9slMLzZxyMKEPYNldJ1AJk
-----END CERTIFICATE-----"
            .to_string(),
    };
    

    let file = std::fs::File::create("/tmp/serde_zip-test.tar").unwrap();
    let writer = serde_archive::ser::tar_writer::TarWriter::new(file);
    assert_eq!(serde_archive::to_writer(writer, &item), Ok(()));
}
