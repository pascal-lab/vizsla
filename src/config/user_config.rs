use itertools::assert_equal;
use serde::{Deserialize, Serialize};
use utils::{json::get_field, paths::Utf8PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FilesWatcherDef {
    Client,
    Notify,
    Server,
}

fn schema(fields: &[(&'static str, &'static str, &str)]) -> serde_json::Value {
    let map = fields
        .iter()
        .map(|(field, ty, default)| {
            let name = field.replace('_', ".");
            let name = format!("rust-analyzer.{name}");
            let props = field_props(field, ty, default);
            (name, props)
        })
        .collect::<serde_json::Map<_, _>>();
    map.into()
}

fn field_props(field: &str, ty: &str, default: &str) -> serde_json::Value {
    let mut map = serde_json::Map::default();

    macro_rules! set {
        ($($key:literal: $value:tt),*$(,)?) => {{$(
            map.insert($key.into(), serde_json::json!($value));
        )*}};
    }

    let default = default.parse::<serde_json::Value>().unwrap();
    set!("default": default);

    match ty {
        "Vec<PathBuf>" => set! {
            "type": "array",
            "items": { "type": "string" },
        },
        "FilesWatcherDef" => set! {
            "type": "string",
            "enum": ["client", "server"],
            "enumDescriptions": [
                "Use the client (editor) to watch files for changes",
                "Use server-side file watching",
            ],
        },
        _ => panic!("missing entry for {ty}: {default}"),
    }

    map.into()
}

macro_rules! default_value {
    ($default:expr, $ty:ty) => {{
        let default_: $ty = $default;
        &serde_json::to_string_pretty(&default_).unwrap()
    }};
}

macro_rules! config_data {
    ($sv:vis struct $name:ident {
         $($(#[doc=$_:literal])*
         $field:ident : $ty:ty = $default:expr,)*
    }) => {
        #[allow(non_snake_case)]
        #[derive(Debug, Clone, PartialEq, Eq)]
        $sv struct $name {
            $($sv $field: $ty,)*
        }

        impl $name {
            $sv fn from_json(mut json: serde_json::Value, error_sink: &mut Vec<(String, serde_json::Error)>) -> $name {
                $name {
                    $( $field: get_field(&mut json, error_sink, stringify!($field), default_value!($default, $ty)), )*
                }
            }

            $sv fn json_schema() -> serde_json::Value {
                schema(&[ $(
                    {
                        (stringify!($field), stringify!($ty), default_value!($default, $ty))
                    },
                )* ])
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field: $default,)*
                }
            }
        }
    };
}

config_data! {
    pub(crate) struct UserConfig {
        /// These directories will be ignored. They are relative to the workspace
        /// root, and globs are not supported. You may also need to add the
        /// folders to Code's `files.watcherExclude`.
        files_excludeDirs: Vec<Utf8PathBuf> = vec![],
        /// Controls file watching.
        files_watcher: FilesWatcherDef = FilesWatcherDef::Client,
        /// Automatically refresh project info on toml changes
        workspace_auto_reload: bool = true,
    }
}

#[test]
fn check_default() {
    let json = serde_json::Value::Null;
    let mut errors = vec![];
    let user_cfg = UserConfig::from_json(json, &mut errors);
    assert_eq!(user_cfg, UserConfig::default());
}
