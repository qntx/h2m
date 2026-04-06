//! Shared test helpers.

#![allow(dead_code)]

use h2m::{Converter, Options};

pub fn with_options(opts: Options) -> Converter {
    Converter::builder()
        .options(opts)
        .use_plugin(h2m::rules::CommonMark)
        .build()
}

pub fn with_domain(domain: &str) -> Converter {
    Converter::builder()
        .use_plugin(h2m::rules::CommonMark)
        .domain(domain)
        .build()
}

pub fn ref_converter(style: h2m::LinkReferenceStyle) -> Converter {
    let mut opts = Options::default();
    opts.link_style = h2m::LinkStyle::Referenced;
    opts.link_reference_style = style;
    with_options(opts)
}
