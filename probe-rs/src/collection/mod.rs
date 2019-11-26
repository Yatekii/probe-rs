pub mod cores;

use std::collections::HashMap;
use std::fs::File;
use std::fs::{self, DirEntry};
use std::io;
use std::io::BufReader;
use std::path::Path;

use crate::config::flash_algorithm::FlashAlgorithm;
use crate::target::Core;
use crate::target::Target;

pub fn get_target(name: impl AsRef<str>) -> Option<Target> {
    let mut map: HashMap<String, Target> = HashMap::new();

    load_targets(
        dirs::home_dir()
            .map(|home| home.join(".config/probe-rs/targets"))
            .as_ref()
            .map(|path| path.as_path()),
        &mut map,
    );

    let name: String = name.as_ref().into();

    map.get(&name.to_ascii_lowercase()).cloned()
}

pub fn get_algorithm(name: impl AsRef<str>) -> Option<FlashAlgorithm> {
    let mut map: HashMap<String, FlashAlgorithm> = HashMap::new();

    let root = dirs::home_dir().map(|home| home.join(".config/probe-rs/algorithms"));
    if let Some(root) = root {
        visit_dirs(root.as_path(), &mut map, &load_algorithms_from_dir).unwrap();

        let name = root
            .join(name.as_ref())
            .as_path()
            .to_string_lossy()
            .to_string();

        map.get(&name).cloned()
    } else {
        log::warn!("Home directory could not be determined while loading algorithms.");
        None
    }
}

pub fn get_core(name: impl AsRef<str>) -> Option<Box<dyn Core>> {
    let map: HashMap<&'static str, Box<dyn Core>> = hashmap! {
        "m0" => Box::new(self::cores::m0::M0) as _,
        "m4" => Box::new(self::cores::m4::M4) as _,
        "m33" => Box::new(self::cores::m33::M33) as _,
    };

    map.get(&name.as_ref().to_ascii_lowercase()[..]).cloned()
}

pub fn load_targets(root: Option<&Path>, map: &mut HashMap<String, Target>) {
    if let Some(root) = root {
        visit_dirs(root, map, &load_targets_from_dir).unwrap();
    } else {
        log::warn!("Home directory could not be determined while loading targets.");
    }
}

pub fn load_targets_from_dir(dir: &DirEntry, map: &mut HashMap<String, Target>) {
    match File::open(dir.path()) {
        Ok(file) => {
            let reader = BufReader::new(file);

            // Read the JSON contents of the file as an instance of `User`.
            match serde_yaml::from_reader(reader) as serde_yaml::Result<Target> {
                Ok(mut target) => {
                    target.name.make_ascii_lowercase();
                    map.insert(target.name.clone(), target);
                }
                Err(e) => log::warn!("Error loading chip definition: {}", e),
            }
        }
        Err(e) => {
            log::info!("Unable to load file {:?}.", dir.path());
            log::info!("Reason: {:?}", e);
        }
    }
}

pub fn load_algorithms_from_dir(dir: &DirEntry, map: &mut HashMap<String, FlashAlgorithm>) {
    match File::open(dir.path()) {
        Ok(file) => {
            let reader = BufReader::new(file);

            // Read the JSON contents of the file as an instance of `User`.
            match serde_yaml::from_reader(reader) as serde_yaml::Result<FlashAlgorithm> {
                Ok(target) => {
                    map.insert(dir.path().to_string_lossy().to_string(), target);
                }
                Err(e) => log::warn!("Error loading chip definition: {}", e),
            }
        }
        Err(e) => {
            log::info!("Unable to load file {:?}.", dir.path());
            log::info!("Reason: {:?}", e);
        }
    }
}

fn visit_dirs<T>(
    dir: &Path,
    map: &mut HashMap<String, T>,
    cb: &dyn Fn(&DirEntry, &mut HashMap<String, T>),
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, map, cb)?;
            } else {
                cb(&entry, map);
            }
        }
    }
    Ok(())
}
