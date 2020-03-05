extern crate handlebars;
use chrono::NaiveDateTime;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use log::debug;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct Templater {
    pub hb: Arc<Handlebars<'static>>,
}
// TODO: implement a render method on this struct to hide the details of the boxed ref

impl Templater {
    pub fn new() -> Templater {
        debug!("Creating template singleton");
        let mut reg = Handlebars::new();
        debug!("Registering template files");
        reg.register_template_file("display.html", "src/templates/display.html")
            .expect("Failed to register display.html");
        reg.register_template_file("healthcheck.html", "src/templates/healthcheck.html")
            .expect("Failed to register healthcheck.html");
        debug!("Registering template helpers");
        reg.register_helper("duration", Box::new(Templater::duration_helper));
        reg.register_helper("systime", Box::new(Templater::systime_helper));
        Templater { hb: Arc::new(reg) }
    }

    fn systime_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .map(|v| v.value())
            .ok_or_else(|| RenderError::new("param not found"))?;
        let systime: NaiveDateTime = serde_json::from_value(param.clone()).unwrap();
        out.write(&systime.format("%c").to_string())?;
        Ok(())
    }

    fn duration_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .map(|v| v.value())
            .ok_or_else(|| RenderError::new("param not found"))?;
        let this_duration: Duration = serde_json::from_value(param.clone()).unwrap();
        let this_in_millis = this_duration.as_secs();
        out.write(&this_in_millis.to_string())?;
        Ok(())
    }
}
