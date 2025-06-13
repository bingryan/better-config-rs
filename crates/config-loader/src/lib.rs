macro_rules! config_feature {
    ($feature:literal, $mod:ident, $trait:ident) => {
        #[cfg(feature = $feature)]
        mod $mod;
        #[cfg(feature = $feature)]
        pub use $mod::$trait;
    };
}

config_feature!("yml", yml, YmlConfig);
config_feature!("json", json, JsonConfig);
config_feature!("toml", toml, TomlConfig);
config_feature!("env", env, EnvConfig);
config_feature!("ini", ini, IniConfig);
