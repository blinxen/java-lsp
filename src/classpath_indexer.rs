use crate::classfile::Classfile;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
};
use zip::ZipArchive;

pub fn index(classpath: &str) -> HashMap<String, Classfile> {
    let mut classes = HashMap::new();

    // for path in classpath.split(":") {
    //     if fs::exists(path).unwrap_or(false) {
    //         if path.ends_with(".jar") {
    //             let mut zip = ZipArchive::new(File::open(path).unwrap()).unwrap();
    //             for index in 0..zip.len() {
    //                 if let Ok(mut file) = zip.by_index(index)
    //                     && file.name().ends_with(".class")
    //                 {
    //                     let mut bytes = vec![0; file.size() as usize];
    //                     file.read_exact(&mut bytes).unwrap();
    //                     if let Some(class) = Classfile::new(&bytes) {
    //                         classes.insert(class.fqdn.to_owned(), class);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    classes
}
