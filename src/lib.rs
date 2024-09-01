use std::{cell::RefCell, rc::Rc};

use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, MediaDeviceInfo, MediaDeviceKind, Navigator, NetworkInformation,
    OfflineAudioCompletionEvent, OfflineAudioContext, OfflineAudioContextOptions, PermissionState,
    PermissionStatus, Permissions, Screen, Window,
};
pub mod webgl;
pub use webgl::*;
pub mod audio_fingerprint;
pub use audio_fingerprint::*;
pub mod canvas;
pub use canvas::*;
pub mod fonts;
pub use fonts::*;
lazy_static::lazy_static! {
    static ref USER_AGENT:String=window().map(|window|window.navigator().user_agent().unwrap_or_default()).unwrap_or_default().to_ascii_lowercase();
}

#[derive(Debug, Clone)]
pub struct FingerPrint {
    pub window_finger_print: Option<WindowFingerPrint>,
    pub audio_finger_print: Option<AudioFingerPrint>,
    pub canvas_finger_print: Option<CanvasFingerPrint>,
    pub webgl_finger_print: Option<WebGLFingerPrint>,
}
impl FingerPrint {
    /// Returns None, if we can't get a web_sys::Window.
    pub async fn new() -> Option<Self> {
        let window = window()?;
        Some(Self {
            window_finger_print: WindowFingerPrint::new(&window).await,
            audio_finger_print: AudioFingerPrint::new(&window).await,
            canvas_finger_print: CanvasFingerPrint::new(&window),
            webgl_finger_print: WebGLFingerPrint::new(&window),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct WindowFingerPrint {
    pub device_pixel_ratio: f64,
    pub screen_finger_print: ScreenFingerPrint,
    pub navigator_finger_print: NavigatorFingerPrint,
    pub indexdb_is_some: bool,
    pub local_storage_is_some: bool,
}
impl WindowFingerPrint {
    pub async fn new(window: &Window) -> Option<Self> {
        let screen_finger_print = ScreenFingerPrint::new(&window.screen().ok()?)?;
        let navigator_finger_print = NavigatorFingerPrint::new(window.navigator()).await?;
        let device_pixel_ratio = window.device_pixel_ratio();
        let indexdb_is_some = window.indexed_db().ok().flatten().is_some();
        let local_storage_is_some = window.local_storage().ok().flatten().is_some();

        Some(Self {
            device_pixel_ratio,
            screen_finger_print,
            navigator_finger_print,
            indexdb_is_some,
            local_storage_is_some,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScreenFingerPrint {
    height: i32,
    width: i32,
    color_depth: i32,
    pixel_depth: i32,
    color_gamut: u32,
    avail_height: i32,
    avail_width: i32,
    avail_top: i32,
    avail_left: i32,
}
impl ScreenFingerPrint {
    pub fn new(screen: &Screen) -> Option<Self> {
        let height = screen.height().ok()?;
        let width = screen.width().ok()?;
        let color_depth = screen.color_depth().ok()?;
        let pixel_depth = screen.pixel_depth().ok()?;
        let color_gamut = screen.color_gamut() as u32;
        let avail_height = screen.avail_height().unwrap_or_default();
        let avail_width = screen.avail_width().unwrap_or_default();
        let avail_top = screen.avail_top().unwrap_or_default();
        let avail_left = screen.avail_left().unwrap_or_default();
        Some(Self {
            height,
            width,
            color_depth,
            pixel_depth,
            color_gamut,
            avail_height,
            avail_width,
            avail_top,
            avail_left,
        })
    }
}
#[derive(Debug, Clone, Default)]
pub struct NavigatorFingerPrint {
    pub network_information: Option<NetworkInformationFingerPrint>,
    pub do_not_track: String,
    pub geolocation_is_ok: bool,
    pub gamepad_ids: Vec<String>,
    pub hardware_concurrency: f64,
    pub language: String,
    pub languages: Vec<String>,
    pub max_touch_points: i32,
    pub audio_input: i32,
    pub audio_output: i32,
    pub video_input: i32,
    pub platform: String,
    pub user_agent: String,
    pub navigator_property_count: usize,
    pub permission_fingerprint: Option<PermissionFingerPrint>,
}

impl NavigatorFingerPrint {
    pub async fn new(navigator: Navigator) -> Option<Self> {
        let user_agent = navigator.user_agent().unwrap_or_default();
        let network_information =
            NetworkInformationFingerPrint::from_result(navigator.connection());
        let do_not_track = String::default(); //navigator.do_not_track(); the fact that this returns a String apparently is panic worthy in chrome.
        let geolocation_is_ok = navigator.geolocation().is_ok();
        let gamepad_ids = navigator
            .get_gamepads()
            .map(|array| {
                array
                    .into_iter()
                    .map(|r| {
                        r.dyn_into::<web_sys::Gamepad>()
                            .ok()
                            .map(|g| g.id())
                            .unwrap_or_default()
                    })
                    .collect::<Vec<String>>()
            })
            .ok()
            .unwrap_or_default();
        let hardware_concurrency = navigator.hardware_concurrency();
        let language = navigator.language().unwrap_or_default();
        let languages = navigator
            .languages()
            .into_iter()
            .map(|l| l.as_string().unwrap_or_default())
            .collect::<Vec<String>>();
        let max_touch_points = navigator.max_touch_points();
        let mut audio_input = 0;
        let mut audio_output = 0;
        let mut video_input = 0;
        if let Some(devices) = navigator.media_devices().ok() {
            if let Some(promise) = devices.enumerate_devices().ok() {
                if let Ok(Ok(array)) = wasm_bindgen_futures::JsFuture::from(promise)
                    .await
                    .map(|result| result.dyn_into::<Array>())
                {
                    for info in array.into_iter() {
                        if let Ok(info) = info.dyn_into::<MediaDeviceInfo>() {
                            match info.kind() {
                                MediaDeviceKind::Audioinput => {
                                    audio_input += 1;
                                }
                                MediaDeviceKind::Audiooutput => {
                                    audio_output += 1;
                                }
                                MediaDeviceKind::Videoinput => {
                                    video_input += 1;
                                }
                                _ => todo!(),
                            }
                        }
                    }
                }
            }
        }
        let platform = navigator.platform().unwrap_or_default();
        let navigator_property_count = Object::keys(&Object::get_prototype_of(&navigator.as_ref()))
            .into_iter()
            .len();
        let permission_fingerprint = PermissionFingerPrint::new(&navigator).await;
        Some(Self {
            network_information,
            do_not_track,
            geolocation_is_ok,
            gamepad_ids,
            hardware_concurrency,
            language,
            languages,
            max_touch_points,
            audio_input,
            audio_output,
            video_input,
            platform,
            user_agent,
            navigator_property_count,
            permission_fingerprint,
        })
    }
}
#[derive(Debug, Clone, Default)]
pub struct PermissionFingerPrint {
    accelerometer: Option<u32>,
    accessibility: Option<u32>,
    ambient_light_sensor: Option<u32>,
    camera: Option<u32>,
    clipboard_read: Option<u32>,
    clipboard_write: Option<u32>,
    geolocation: Option<u32>,
    background_sync: Option<u32>,
    magnetometer: Option<u32>,
    microphone: Option<u32>,
    midi: Option<u32>,
    notifications: Option<u32>,
    payment_handler: Option<u32>,
    persistent_storage: Option<u32>,
    push: Option<u32>,
}

impl PermissionFingerPrint {
    pub async fn new(navigator: &Navigator) -> Option<Self> {
        let permissions = &navigator.permissions().ok()?;
        let accelerometer = query_permission(permissions, "accelerometer")
            .await
            .map(|r| r as u32);
        let accessibility = query_permission(permissions, "accessibility")
            .await
            .map(|r| r as u32);
        let ambient_light_sensor = query_permission(permissions, "ambient-light-sensor")
            .await
            .map(|r| r as u32);
        let camera = query_permission(permissions, "camera")
            .await
            .map(|r| r as u32);
        let clipboard_read = query_permission(permissions, "clipboard-read")
            .await
            .map(|r| r as u32);
        let clipboard_write = query_permission(permissions, "clipboard-write")
            .await
            .map(|r| r as u32);
        let geolocation = query_permission(permissions, "geolocation")
            .await
            .map(|r| r as u32);
        let background_sync = query_permission(permissions, "background-sync")
            .await
            .map(|r| r as u32);
        let magnetometer = query_permission(permissions, "magnetometer")
            .await
            .map(|r| r as u32);
        let microphone = query_permission(permissions, "microphone")
            .await
            .map(|r| r as u32);
        let midi = query_permission(permissions, "midi")
            .await
            .map(|r| r as u32);
        let notifications = query_permission(permissions, "notifications")
            .await
            .map(|r| r as u32);
        let payment_handler = query_permission(permissions, "payment-handler")
            .await
            .map(|r| r as u32);
        let persistent_storage = query_permission(permissions, "persistent-storage")
            .await
            .map(|r| r as u32);
        let push = query_permission(permissions, "push")
            .await
            .map(|r| r as u32);

        Some(Self {
            accelerometer,
            accessibility,
            ambient_light_sensor,
            camera,
            clipboard_read,
            clipboard_write,
            geolocation,
            background_sync,
            magnetometer,
            microphone,
            midi,
            notifications,
            payment_handler,
            persistent_storage,
            push,
        })
    }
}

async fn query_permission(
    permissions_api: &Permissions,
    permission: &str,
) -> Option<PermissionState> {
    let object = js_sys::Object::new();
    Reflect::set(&object, &"name".into(), &permission.into()).ok()?;

    let promise = permissions_api.query(&object).ok()?;
    let permission_state = JsFuture::from(promise)
        .await
        .ok()?
        .dyn_into::<PermissionStatus>()
        .ok()?
        .state();

    Some(permission_state)
}

#[derive(Debug, Clone, Default)]
pub struct NetworkInformationFingerPrint {
    downlink: Option<f64>,
    downlink_max: Option<f64>,
    effective_type: Option<String>,
    rtt: Option<f64>,
    save_data: Option<bool>,
    type_: Option<String>,
}
impl NetworkInformationFingerPrint {
    pub fn from_result(n: Result<NetworkInformation, JsValue>) -> Option<Self> {
        let network = n.ok()?;
        let downlink = js_sys::Reflect::get(&network, &"downlink".into())
            .ok()
            .map(|r| r.as_f64())
            .flatten();
        let downlink_max = js_sys::Reflect::get(&network, &"downlinkMax".into())
            .ok()
            .map(|r| r.as_f64())
            .flatten();
        let effective_type = js_sys::Reflect::get(&network, &"effectiveType".into())
            .ok()
            .map(|r| r.as_string())
            .flatten();
        let rtt = js_sys::Reflect::get(&network, &"rtt".into())
            .ok()
            .map(|r| r.as_f64())
            .flatten();
        let save_data = js_sys::Reflect::get(&network, &"saveData".into())
            .ok()
            .map(|r| r.as_bool())
            .flatten();
        let type_ = js_sys::Reflect::get(&network, &"type".into())
            .ok()
            .map(|r| r.as_string())
            .flatten();

        Some(Self {
            downlink,
            downlink_max,
            effective_type,
            rtt,
            save_data,
            type_,
        })
    }
}
