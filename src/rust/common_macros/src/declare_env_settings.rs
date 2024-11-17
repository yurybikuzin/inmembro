#[macro_export]
macro_rules! declare_env_settings {
    (
        $( $config_fields:ident $($option:ident)? : $config_types:ty, )*
    ) => {
        declare_env_settings!(@parse {}, {}, {};
            $( $config_fields $($option)? : $config_types, )*
        );
    };
    (@parse
        { $($config_body:tt)* },
        { $($init_body:tt)* },
        { $($field_list_body:tt)* };

        $config_field:ident Option : $config_type:ty,
        $( $config_fields:ident $($option:ident)? : $config_types:ty, )*
    ) => {
        paste::paste! {
            declare_env_settings!(@parse
                { $($config_body)*
                    pub $config_field: Option<$config_type>,
                },
                { $($init_body)*
                    declare_env_settings!(@let_opt_from_env $config_field, [< $config_field:upper >], $config_type);
                },
                { $($field_list_body)*
                    $config_field,
                };
                $( $config_fields $($option)? : $config_types, )*
            );
        }
    };
    (@parse
        { $($config_body:tt)* },
        { $($init_body:tt)* },
        { $($field_list_body:tt)* };

        $config_field:ident: $config_type:ty,
        $( $config_fields:ident $($option:ident)?: $config_types:ty, )*
    ) => {
        paste::paste! {
            declare_env_settings!(@parse
                { $($config_body)*
                    pub $config_field: $config_type,
                },
                { $($init_body)*
                    declare_env_settings!(@let_from_env $config_field, [< $config_field:upper >], $config_type);
                },
                { $($field_list_body)*
                    $config_field,
                };
                $( $config_fields $($option)? : $config_types, )*
            );
        }
    };
    (@parse
        { $($config_body:tt)* },
        { $($init_body:tt)* },
        { $($field_list_body:tt)* };
    ) => {
        #[derive(Clone, Debug, Default)]
        pub struct EnvSettings {
            $($config_body)*
        }
        impl EnvSettings {
            pub fn init() -> anyhow::Result<Self> {
                $($init_body)*
                Ok(Self {
                    $($field_list_body)*
                })
            }
        }
        lazy_static::lazy_static! {
            pub static ref ENV_SETTINGS: std::sync::RwLock<EnvSettings> = std::sync::RwLock::new(
                EnvSettings::init().map_err(|err| anyhow!("failed to init EnvSettings: {err}")).unwrap()
            );
        }
    };

    // --------------------------------

    (@let_from_env $var: ident, $env: ident, String) => {
        declare_env_settings!(@let_from_env $var, $env);
    };
    (@let_from_env $var: ident, $env: ident, $type: ty) => {
        declare_env_settings!(@let_from_env $var, $env);
        let $var = $var.parse::<$type>().context(format!(
            "{}: failed to {:?}.parse::<{}>",
            stringify!($env),
            $var,
            stringify!($type),
        ))?;
    };
    (@let_from_env $var: ident, $env: ident) => {
        let $var =
            std::env::var(stringify!($env)).context(format!("{} required", stringify!($env)))?;
    };

    // --------------------------------

    (@let_opt_from_env $var: ident, $env: ident, String) => {
        declare_env_settings!(@let_opt_from_env $var, $env);
    };
    (@let_opt_from_env $var: ident, $env: ident, $type: ty) => {
        declare_env_settings!(@let_opt_from_env $var, $env);
        let $var = if let Some($var) = $var {
            Some($var.parse::<$type>().context(format!(
                "{}: failed to {:?}.parse::<{}>",
                stringify!($env),
                $var,
                stringify!($type),
            ))?)
        } else {
            None
        };
    };
    (@let_opt_from_env $var: ident, $env: ident) => {
        let $var =
            std::env::var(stringify!($env)).ok();
    };
}

#[macro_export]
macro_rules! declare_env_settings_for_server {
    (
        $( $config_fields:ident $($option:ident)? : $config_types:ty, )*
    ) => {
        declare_env_settings!(@parse {}, {}, {};
            $( $config_fields $($option)? : $config_types, )*
            port: u16,
            port_dev: u16,
            port_demo: u16,
            port_rc: u16,
            port_local: u16,
        );
        impl EnvSettings {
            pub fn set_port(port: Option<u16>, op_mode: op_mode::OpMode) {
                match op_mode {
                    op_mode::OpMode::Prod => {
                        env_settings!(port = port);
                    }
                    op_mode::OpMode::Dev => {
                        env_settings!(port_dev = port);
                    }
                    op_mode::OpMode::Demo => {
                        env_settings!(port_demo = port);
                    }
                    op_mode::OpMode::Rc => {
                        env_settings!(port_rc = port);
                    }
                    op_mode::OpMode::Local => {
                        env_settings!(port_local = port);
                    }
                }
            }
            pub fn port(op_mode: op_mode::OpMode) -> u16 {
                match op_mode {
                    op_mode::OpMode::Prod => {
                        env_settings!(port)
                    }
                    op_mode::OpMode::Dev => {
                        env_settings!(port_dev)
                    }
                    op_mode::OpMode::Demo => {
                        env_settings!(port_demo)
                    }
                    op_mode::OpMode::Rc => {
                        env_settings!(port_rc)
                    }
                    op_mode::OpMode::Local => {
                        env_settings!(port_local)
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! env_settings {
    ($field:ident) => {
        (*ENV_SETTINGS.read().unwrap()).$field
    };
    ($name:ident = $from:expr) => {
        paste::paste!{
            if let Some($name) = $from {
                if env_settings!($name) != $name {
                    warn!(
                        concat!("will use ", stringify!($name), " {:?} by opt instead of ", stringify!([< $name:upper >])," = {:?} in .env"),
                        $name,
                        env_settings!($name)
                    );
                    (*ENV_SETTINGS.write().unwrap()).$name = $name;
                }
            };
        }
    }
}
