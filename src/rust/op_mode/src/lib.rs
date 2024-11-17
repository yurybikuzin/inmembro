#[allow(unused_imports)]
use anyhow::{anyhow, bail, Context, Error, Result};
#[allow(unused_imports)]
use tracing::{debug, error, info, span, trace, warn, Level};

use serde::{Deserialize, Serialize};
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, strum::Display, strum::EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum OpMode {
    #[strum(serialize = "prod")]
    Prod = 0,
    #[strum(serialize = "dev")]
    Dev = 1,
    #[strum(serialize = "demo")]
    Demo = 2,
    #[strum(serialize = "rc")]
    Rc = 3,
    #[strum(serialize = "local")]
    Local = 4,
}
common_macros::r#impl!(FromStr for OpMode; strum);

impl Default for OpMode {
    fn default() -> Self {
        Self::Prod
    }
}

impl OpMode {
    // pub fn login2_url_start(&self) -> String {
    //     let domain = "id.z9v.ru";
    //     let pkg_name = "login2_back";
    //     let is_local = matches!(self, OpMode::Local);
    //     format!(
    //         "https://{}{}{}/{}",
    //         if !is_local { "" } else { "local." },
    //         domain,
    //         match self {
    //             OpMode::Prod | OpMode::Local => "",
    //             OpMode::Dev => "/dev",
    //             OpMode::Demo => "/demo",
    //             OpMode::Rc => "/rc",
    //         },
    //         pkg_name
    //     )
    // }
    pub fn get_actual(op_mode_by_cmd_line: Option<Self>) -> Self {
        let op_mode_by_current_exe = OpMode::from_current_exe()
            .map_err(|err| {
                warn!("{}:{}: {err}", file!(), line!());
                err
            })
            .unwrap_or_default();

        if let Some(op_mode) = if let Some(op_mode_by_cmd_line) = op_mode_by_cmd_line {
            if op_mode_by_current_exe == op_mode_by_cmd_line {
                None
            } else {
                info!("op_mode: {op_mode_by_current_exe} is overrided by '--op-mode {op_mode_by_cmd_line}'");
                Some(op_mode_by_cmd_line)
            }
        } else {
            None
        } {
            op_mode
        } else {
            info!("op_mode: {}", op_mode_by_current_exe);
            op_mode_by_current_exe
        }
    }
    // pub fn fingerprint_url(&self, id_domain: &str, fingerprint_path: &str) -> String {
    //     format!(
    //         "https://{}{}{}/{}",
    //         if matches!(self, Self::Local) {
    //             "local."
    //         } else {
    //             ""
    //         },
    //         id_domain, // id.z9v.ru
    //         self.route_prefix(),
    //         fingerprint_path // login2_back/fingerprint.js
    //     )
    // }
    pub fn ws_url(&self, host: &str, back: &str) -> String {
        format!("wss://{}{}/{}/ws/", host, self.route_prefix(), back)
    }
    pub fn route_prefix(&self) -> &'static str {
        match self {
            OpMode::Prod | OpMode::Local => "",
            OpMode::Dev => "/dev",
            OpMode::Demo => "/demo",
            OpMode::Rc => "/rc",
        }
    }
    // pub fn from_href(href: &str) -> Self {
    //     match domain_op_mode_route(href) {
    //         Ok((_, op_mode, _)) => op_mode,
    //         Err(err) => panic!("{err}"),
    //     }
    // }
    fn from_path_segment(s: &str) -> Self {
        match s {
            "target" => OpMode::Local,
            "rc" => OpMode::Rc,
            "demo" => OpMode::Demo,
            "dev" => OpMode::Dev,
            _ => OpMode::Prod,
        }
    }
    pub fn from_current_exe() -> Result<Self> {
        let current_exe = std::env::current_exe()?;
        current_exe
            .components()
            .rev()
            .nth(2)
            .and_then(|i| {
                if let std::path::Component::Normal(s) = i {
                    Some(s)
                } else {
                    None
                }
            })
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("failed to obtain op_mode from {:?}", current_exe))
            .map(Self::from_path_segment)
    }
}

// pub fn domain_op_mode_route(href: &str) -> Result<(&str, OpMode, &str)> {
//     if let Some(cap) = RE_HREF.captures(href) {
//         let domain = cap.name("host").map(|value| value.as_str()).unwrap();
//         let pathname_opt = cap.name("pathname").map(|value| value.as_str());
//         if let Some(domain) = domain.strip_prefix("local.") {
//             Ok((domain, OpMode::Local, pathname_opt.unwrap_or("")))
//         } else {
//             let (op_mode, route) = if let Some(pathname) = pathname_opt {
//                 if let Some(route) = pathname.strip_prefix("dev/") {
//                     (OpMode::Dev, route)
//                 } else if let Some(route) = pathname.strip_prefix("demo/") {
//                     (OpMode::Demo, route)
//                 } else if let Some(route) = pathname.strip_prefix("rc/") {
//                     (OpMode::Rc, route)
//                 } else {
//                     (OpMode::Prod, "")
//                 }
//             } else {
//                 (OpMode::Prod, "")
//             };
//             Ok((domain, op_mode, route))
//         }
//     } else {
//         Err(anyhow!("href={href:?} does not match {RE_HREF_STR}"))
//     }
// }
//
// lazy_static::lazy_static! {
//     pub static ref RE_HREF: regex::Regex = {
//         match regex::Regex::new(RE_HREF_STR) {
//             Ok(ret) => ret,
//             Err(err) => panic!("{}:{}: {err}", file!(), line!()),
//         }
//     };
// }
//
// pub const RE_HREF_STR: &str = r"(?x)
//     ^
//         https?://
//             (?P<host> [^/?\#]+ )
//         (?: /
//             (?P<pathname> [^?\#]* )
//         )?
//         (?: [?]
//             (?P<query> [^\#]* )
//         )?
//         (?: \#
//             (?P<hash> .* )
//         )?
//     $
//     ";
