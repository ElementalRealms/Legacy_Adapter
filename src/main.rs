/*
    Developer note: The zip format is currently not being supported due to security concerns(may be supported later on)
*/

extern crate json;
extern crate mysql;
use mysql::Pool;
use std::env::{args, current_dir};
use std::io::{self, Write};
use std::{path::Path, vec::Vec};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Some(arg1) = args().nth(1) {
        //Gets database name from mysql string
        let mut db_name = String::new();
        for chars in arg1.chars() {
            if (chars == '/') {
                db_name = String::new();
            } else {
                db_name.push(chars);
            }
        }
        println!("ER Legacy Adapter Version {}", CURRENT_VERSION);
        //DEBUG
        println!("{}", arg1);

        //DEBUG
        //  mysql://ermlpublicread:hmDmxuhheilgKXUWTjzC@db.elementalrealms.net/ElementalRealms

        let str_path = args().nth(2).unwrap_or(".".to_string());
        //add /ER_MySQL_versions.json when exporting
        //DEBUG
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
        let mut _mysql_legacy: Vec<MysqlLegacy> = vec![];
        match Pool::new(&arg1){
            Ok(pool)=> {
                _mysql_legacy = pool.prep_exec("SELECT Version_UID, Config, Biome, Script, Badge, Forge, Mods, Client, Server, Visable, Dev FROM Version ORDER BY Version_UID", ())
                .map(|result|{
                    result.map(|x| x.unwrap()).map(|row|{
                        let (version, config, biome, script, badge, forge, stringmods, client, server, visable, dev): (_,_,_,_,_,_,String,_,_,bool,bool)= mysql::from_row(row);
                        let mut mods: Vec<ModEr> = vec![];
                        for modname in stringmods.split(","){
                            for _mod in _mods_all.iter(){
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
        //End of MySQL

        let mut json_export = json::JsonValue::new_object();
        for mysqlv in _mysql_legacy {
            json_export["versions"][&mysqlv.version]["global"]["git"] = json::JsonValue::new_array();
            match git_url(&mysqlv.config, &db_name, "MC_Badge".to_string()){
                    Ok(a) => {
                            json_export["versions"][&mysqlv.version]["global"]["git"].push(a).unwrap();
                        },
                    Err(_) => println!("Failed to add:{} to {}", &mysqlv.config, &mysqlv.version)
                }
            json_export["versions"][&mysqlv.version]["global"]["wget"] = json::JsonValue::new_array();
            if mysqlv.biome != "null" {
                //json_export["versions"][&mysqlv.version]["global"]["zip"].push(git_url(mysqlv.biome, db_name));
            }
        }

        //the logic behind this function is ported from the legacy client for consistency
        fn git_url(in_url: &String, db_name: &String, repo : String) -> Result<json::JsonValue, bool> {
            let mut used_url = json::JsonValue::new_object();
            used_url["url"] = in_url.clone().into();
            if !(in_url.starts_with("http") || in_url.starts_with("https")) 
            {
                if in_url.to_lowercase().contains("github.com"){
                        //TODO Complex return tree stuff
                }else{
                    used_url["url"][0] = format!("https://github.com/{}/{}.git", &db_name, repo).into();
                    used_url["commit"] = in_url.clone().into();
                    return Ok(used_url);
                }
                used_url["url"] = format!("https://github.com/{}/{}", db_name, &in_url).into();
            }
            Err(true)
        }

        //DEBUG
        println!("{:#}", json_export);

        /* Version name print
        for valu in json_export["versions"].entries() {
            let (v_name, _) = valu;
            println!("{:?}", v_name);
        }
        */
    } else {
        println!("Must be URL encoded,\n accepted format:\n mysql://USERNAME:PASSWORD@IP/DATABASE_NAME [EXPORT_FOLDER_PATH]");
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
