/*
    Developer note: The zip format is currently not being supported due to security concerns and priorities(may be supported later on)
*/

extern crate json;
extern crate mysql;
use json::object;
use mysql::Pool;
use std::env::args;
use std::{fs, vec::Vec};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("ER Legacy Adapter Version {}", CURRENT_VERSION);

    if let Some(arg1) = args().nth(1) {
        //Gets database name from mysql string
        let mut db_name = String::new();
        for chars in arg1.chars() {
            if chars == '/' {
                db_name = String::new();
            } else {
                db_name.push(chars);
            }
        }

        //Argument handler
        let mut debug = false;
        let mut export_mods: Option<String> = None;
        let mut export_versions: Option<String> = None;
        let mut mode = vec![vec![true, false]];
        for arg_num in 2..args().count() {
            match &args().nth(arg_num).unwrap() as &str {
                "Em" => export_mods = args().nth(arg_num + 1),
                "Ev" => export_versions = args().nth(arg_num + 1),
                "forge" | "Forge" => {} //TODO maybe one day
                "PS" | "ps" => {
                    debug = true;
                }
                "mode" | "Mode" => {
                    mode = Vec::new();
                    for _mode in args().nth(arg_num + 1).unwrap().split('-') {
                        mode.push(vec![
                            _mode.chars().nth(0).unwrap() == '1',
                            _mode.chars().nth(1).unwrap() == '1',
                        ])
                    }
                }
                _ => {}
            }
        }

        if debug {
            println!("Enabled debug text");
        }
        //NOTE
        //  mysql://ermlpublicread:hmDmxuhheilgKXUWTjzC@db.elementalrealms.net/ElementalRealms

        let mut _mods_all: Vec<ModEr> = vec![];
        match Pool::new(&arg1) {
            Ok(pool) => {
                _mods_all = pool
                    .prep_exec("SELECT FileName, URL FROM Mods ORDER BY FileName", ())
                    .map(|result| {
                        result
                            .map(|x| x.unwrap())
                            .map(|row| {
                                let (name, url) = mysql::from_row(row);
                                ModEr { name, url }
                            })
                            .collect()
                    })
                    .unwrap()
            }
            Err(a) => {
                println!("{}", a);
            }
        }
        if debug {
            println!("Obtained mods");
        }
        if let Some(ex_mods) = export_mods {
            let mut json_data = json::JsonValue::new_array();
            if !_mods_all[0].is_empty() {
                for _mod in &_mods_all {
                    let mut data = json::JsonValue::new_object();
                    data["name"] = _mod.name.clone().into();
                    data["url"] = _mod.url.clone().into();
                    json_data.push(data).unwrap();
                }
                fs::write(&ex_mods, json_data.dump()).expect("Failed to export mods");
            }
            if debug {
                println!("Wrote mods to: {}", ex_mods);
            }
        }
        let mut _mysql_legacy: Vec<MysqlLegacy> = vec![];
        match Pool::new(&arg1){
            Ok(pool)=> {
                _mysql_legacy = pool.prep_exec("SELECT Version_UID, Config, Biome, Script, Badge, Forge, Mods, Client, Server, Visable, Dev FROM Version ORDER BY Version_UID", ())
                .map(|result|{
                    result.map(|x| x.unwrap()).map(|row|{
                        let (version, config, biome, script, badge, forge, stringmods, client, server, visable, dev): (_,_,_,_,_,_,String,_,_,bool,bool)= mysql::from_row(row);
                        let mut mods: Vec<ModEr> = vec![];
                        for modname in stringmods.split(","){
                            for _mod in &_mods_all{
                                if _mod.name == modname{
                                    mods.push(ModEr{name: _mod.name.trim().to_string(), url: _mod.url.trim().to_string()});
                                }
                            }
                        }
                        MysqlLegacy{version, config, biome, script, badge, forge, mods, client, server, visable, dev}
                    }).collect()
                }).unwrap()
            },
            Err(a)=>  {
                println!("{}", a);
            }
        }
        if debug {
            println!("Obtained versions");
        }
        //End of MySQL

        let mut json_export = json::JsonValue::new_object();
        for mysqlv in _mysql_legacy {
            if !mode.contains(&vec![mysqlv.visable, mysqlv.dev]) {
                continue;
            }
            json_export["mc"]["version"][&mysqlv.version]["global"] = json::JsonValue::new_array();

            if !(&mysqlv.config == "null" || &mysqlv.config == "") {
                match git_url(mysqlv.config.clone(), &db_name, "MC_Configs".to_string()) {
                    Ok(a) => {
                        json_export["mc"]["version"][&mysqlv.version]["global"]
                            .push(a)
                            .unwrap();
                    }
                    Err(_) => println!("Failed to add:{} to {}", &mysqlv.config, &mysqlv.version),
                }
            }

            if !(&mysqlv.biome == "null" || &mysqlv.biome == "") {
                match git_url(mysqlv.biome.clone(), &db_name, "MC_Biome".to_string()) {
                    Ok(a) => {
                        json_export["mc"]["version"][&mysqlv.version]["global"]
                            .push(a)
                            .unwrap();
                    }
                    Err(_) => println!("Failed to add:{} to {}", &mysqlv.biome, &mysqlv.version),
                }
            }
            if !(&mysqlv.script == "null" || &mysqlv.script == "") {
                match git_url(mysqlv.script.clone(), &db_name, "MC_Script".to_string()) {
                    Ok(a) => {
                        json_export["mc"]["version"][&mysqlv.version]["global"]
                            .push(a)
                            .unwrap();
                    }
                    Err(_) => println!("Failed to add:{} to {}", &mysqlv.script, &mysqlv.version),
                }
            }
            if !(&mysqlv.badge == "null" || &mysqlv.badge == "") {
                match git_url(mysqlv.badge.clone(), &db_name, "MC_Badge".to_string()) {
                    Ok(a) => {
                        json_export["mc"]["version"][&mysqlv.version]["global"]
                            .push(a)
                            .unwrap();
                    }
                    Err(_) => println!("Failed to add:{} to {}", &mysqlv.badge, &mysqlv.version),
                }
            }
            //NOTE Mods download
            //json_export["mc"]["version"][&mysqlv.version]["global"]["wget"] =json::JsonValue::new_array();

            //TODO UNTESTED
            for _mod in &mysqlv.mods {
                json_export["mc"]["version"][&mysqlv.version]["global"]
                    .push(json::array![
                        json::object! {"url"=>_mod.url.clone(),"path"=>format!("mods/{}",&_mod.name)}
                    ])
                    .unwrap();
            }
            //TODO UNTESTED
            for _mod in mysqlv.server.split(',') {
                for _mod_all in &_mods_all {
                    if _mod_all.name == _mod {
                        json_export["mc"]["version"][&mysqlv.version]["server"].push(json::array![
                        json::object! {"url"=>_mod_all.url.clone() as String,"path"=>format!("mods/{}",&_mod_all.name)}]).unwrap();
                    }
                }
            }
            //TODO UNTESTED
            for _mod in mysqlv.client.split(',') {
                let mut used_mod = json::JsonValue::new_object();

                if _mod.ends_with("0") || _mod.ends_with("1") {
                    let mut changed_mod = _mod.to_string();
                    changed_mod.pop();
                    used_mod["path"] = format!("mods/{}", changed_mod).into();
                } else {
                    used_mod["enable"] = false.into();
                    used_mod["path"] = format!("mods/{}", &_mod).into();
                }

                for _mod_all in &_mods_all {
                    if _mod_all.name == _mod {
                        used_mod["url"] = _mod_all.url.clone().into();
                    }
                }
                json_export["mc"]["version"][&mysqlv.version]["client"]
                    .push(json::array![used_mod])
                    .unwrap();
            }

            if debug {
                println!("Parsed {}", &mysqlv.version);
            }
        }
        if let Some(ex_versions) = export_versions {
            fs::write(&ex_versions, json_export.dump()).expect("Failed to export versions");
            if debug {
                println!("Wrote versions to: {}", ex_versions);
            }
        }

        //the logic behind this function is ported from the legacy client for consistency
        fn git_url(
            mut in_url: String,
            db_name: &String,
            repo: String,
        ) -> Result<json::JsonValue, bool> {
            let mut used_url = json::JsonValue::new_object();
            used_url["url"][0] = format!("https://github.com/{}/{}.git", &db_name, &repo).into();
            used_url["function"] = "git".into();
            match &repo as &str {
                "MC_Configs" => used_url["path"] = "/config".into(),
                "MC_Biome" => used_url["path"] = "/config/TerrainControl".into(),
                "MC_Script" => used_url["path"] = "/scripts".into(),
                _ => {}
            }
            if in_url.to_lowercase().contains("github.com") {
                in_url = in_url
                    .clone()
                    .split_off(in_url.rfind("/archive/").unwrap() + 9);
                in_url.pop();
                in_url.pop();
                in_url.pop();
                in_url.pop();
                used_url["commit"] = in_url.clone().into();
                return Ok(json::array![used_url]);
            } else {
                if !(in_url.starts_with("http") || in_url.starts_with("https")) {
                    used_url["commit"] = in_url.clone().into();
                    return Ok(json::array![used_url]);
                }
            }
            Err(true)
        }

        //NOTE
        println!("{:#}", json_export);

        /*NOTE
        for valu in json_export["versions"].entries() {
            let (v_name, _) = valu;
            println!("{:?}", v_name);
        }
        */
        if debug {
            println!("DONE");
        }
    } else {
        println!("\nMust be URL encoded\n accepted format:\n mysql://USERNAME:PASSWORD@IP/DATABASE_NAME [ARGS]\n  Em [file path]  exports a list of all mods\n  Ev [file path]    exports versions\n  forge [link] will assign this version of forge to every version\n  PS a simplistic debug parameter that gives a indication of what the program is doing\n  mode 00-10-.. sets what versions should be included in export\n   10 (Visable: true Dev- false) is the defautl.\n   for exsample if you also wanted all dev versions: mode 10-11\n");
    }
}

struct MysqlLegacy {
    version: String,
    config: String,
    biome: String,
    script: String,
    badge: String,
    forge: String,
    mods: Vec<ModEr>,
    client: String,
    server: String,
    visable: bool,
    dev: bool,
}

struct ModEr {
    name: String,
    url: String,
}
impl ModEr {
    fn is_empty(&self) -> bool {
        self.name.is_empty() || self.url.is_empty()
    }
}
impl PartialEq for ModEr {
    fn eq(&self, other: &ModEr) -> bool {
        (self.name == other.name) & (self.url == other.url)
    }
}
