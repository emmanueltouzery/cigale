use core::time::Duration;
use std::io::Read;
use yaml_rust::{Yaml, YamlLoader};

fn main() {
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(30))
        .connection_verbose(true)
        .build()
        .unwrap();

    println!("Fetching the redmine source...");
    let zip_bytes = client
        .get("https://github.com/redmine/redmine/archive/master.zip")
        .send()
        .unwrap()
        .error_for_status()
        .unwrap()
        .bytes()
        .unwrap();

    println!("Unzipping the redmine source...");
    let reader = std::io::Cursor::new(zip_bytes);
    let mut zip = zip::ZipArchive::new(reader).unwrap();

    let locale_files: Vec<String> = zip
        .file_names()
        .filter(|fname| {
            fname.starts_with("redmine-master/config/locales/") && fname.ends_with(".yml")
        })
        .map(|s| s.to_string()) // TODO annoying to clone here, but otherwise i borrow at the same as the iterator
        .collect();

    let mut contents = String::new();
    for locale_file in locale_files {
        let mut file = zip.by_name(&locale_file).unwrap();
        file.read_to_string(&mut contents).unwrap();
        let yaml = YamlLoader::load_from_str(&contents)
            .unwrap()
            .pop() // only one element
            .unwrap()
            .into_hash()
            .unwrap();
        let locale_name = yaml.keys().next().unwrap().as_str().unwrap();
        let date_format = yaml[&Yaml::from_str(locale_name)].as_hash().unwrap()
            [&Yaml::from_str("date")]
            .as_hash()
            .unwrap()[&Yaml::from_str("formats")]
            .as_hash()
            .unwrap()[&Yaml::from_str("default")]
            .as_str()
            .unwrap();
        let today_translation = yaml[&Yaml::from_str(locale_name)].as_hash().unwrap()
            [&Yaml::from_str("label_today")]
            .as_str()
            .unwrap();
        println!(
            "locales.insert(\"{}\", LocaleInfo::new(\"{}\", \"{}\"));",
            locale_name, date_format, today_translation
        );
        contents.clear();
    }
}
