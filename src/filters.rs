use crate::{generator::MIME_SVG, STATE};
use qrcode::render::svg;
use qrcode::QrCode;
use std::{collections::HashMap, io::Read};
use tera::{try_get_value, Error, Result, Tera, Value};

pub fn register(tera: &mut Tera) {
    tera.register_filter("fetch", fetch);
    tera.register_filter("to_qr", to_qr);
}

fn fetch(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let raw_url = try_get_value!("fetch", "value", String, value);
    let create_err = |msg: &str| Error::msg(format!("Fail to fetch `{}`, {}", &raw_url, msg));

    if !STATE.read().unwrap().guard_url(&raw_url) {
        return Err(create_err("invalid url"));
    }

    let resp = ureq::get(&raw_url)
        .call()
        .map_err(|e| create_err(&e.to_string()))?;

    let status = resp.status();
    if status >= 300 {
        return Err(create_err(&format!("status `{}` is not ok", status)));
    }
    let mime = resp
        .header("Content-Type")
        .ok_or_else(|| create_err("no content-type header"))?
        .to_owned();

    let mut bytes: Vec<u8> = match resp.header("Content-Length").and_then(|v| v.parse().ok()) {
        Some(len) => Vec::with_capacity(len),
        None => Vec::new(),
    };

    resp.into_reader()
        .read_to_end(&mut bytes)
        .map(|e| create_err(&e.to_string()))?;

    Ok(Value::String(to_dataurl(&mime, &bytes)))
}

fn to_qr(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("to_qr", "value", String, value);
    let code = QrCode::new(s).unwrap();
    let svg_xml = code.render::<svg::Color>().build();
    Ok(Value::String(to_dataurl(MIME_SVG, svg_xml.as_bytes())))
}

fn to_dataurl(mime: &str, value: &[u8]) -> String {
    let b64 = base64::encode(value);
    format!("data:{};base64,{}", mime, b64)
}
