use crate::utils;
use fluid_parser::ast::*;
use std::fmt::Write;
use std::sync::atomic;
use std::sync::Mutex;

static COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
static I18N: atomic::AtomicBool = atomic::AtomicBool::new(false);
static LAST_MENU: Mutex<String> = Mutex::new(String::new());

pub const ALLOWS: &str = r#"// Automatically generated by fl2rust

#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::needless_update)]"#;

const HEADER: &str = r#"
use fltk::browser::*;
use fltk::button::*;
use fltk::dialog::*;
use fltk::enums::*;
use fltk::frame::*;
use fltk::group::*;
use fltk::image::*;
use fltk::input::*;
use fltk::menu::*;
use fltk::misc::*;
use fltk::output::*;
use fltk::prelude::*;
use fltk::table::*;
use fltk::text::*;
use fltk::tree::*;
use fltk::valuator::*;
use fltk::widget::*;
use fltk::window::*;"#;

fn i18nize(s: &str) -> String {
    if I18N.load(atomic::Ordering::Relaxed) {
        format!("&tr!(\"{}\")", s)
    } else {
        format!("\"{}\"", s)
    }
}

fn is_parent_type(typ: &str) -> bool {
    matches!(
        typ,
        "Window"
            | "Group"
            | "Pack"
            | "Tabs"
            | "Scroll"
            | "Table"
            | "Tile"
            | "Wizard"
            | "MenuBar"
            | "MenuButton"
            | "Choice"
            | "Flex"
    )
}

fn is_menu_type(typ: &str) -> bool {
    matches!(typ, "MenuBar" | "SysMenuBar" | "MenuButton" | "Choice")
}

fn add_menus(widgets: &[Widget], sub: &mut Vec<String>) -> String {
    let mut wid = String::new();
    let mut substyle = String::new();
    for w in widgets {
        if w.typ == "MenuItem" {
            wid += "\tlet idx = ";
            {
                wid += &*LAST_MENU.lock().unwrap();
            }
            wid += ".add_choice(";
            let mut temp = String::new();
            temp += &sub.iter().map(|x| x.to_owned() + "/").collect::<String>();
            temp += w.props.label.as_ref().unwrap_or(&String::new());
            wid += &i18nize(&temp);
            wid += ");\n";

            let name = &format!("{}.at(idx).unwrap()", *LAST_MENU.lock().unwrap());
            if let Some(v) = &w.props.shortcut {
                writeln!(
                    wid,
                    "\t{}.set_shortcut(unsafe {{std::mem::transmute({})}});",
                    name, v
                )
                .unwrap();
            }
            if let Some(v) = &w.props.typ {
                writeln!(wid, "\t{}.set_flag(MenuFlag::{});", name, v).unwrap();
            } else if w.props.divider.is_some() {
                writeln!(wid, "\t{}.set_flag(MenuFlag::MenuDivider);", name).unwrap();
            }
            if let Some(v) = &w.props.callback {
                writeln!(wid, "\t{}.set_callback({});", name, v).unwrap();
            }
            if let Some(v) = &w.props.labeltype {
                let temp = utils::global_to_pascal(v);
                let temp = if temp == "No" { "None" } else { temp.as_str() };
                writeln!(wid, "\t{}.set_label_type(LabelType::{});", name, temp).unwrap();
            }
            if let Some(v) = &w.props.labelfont {
                writeln!(wid, "\t{}.set_label_font(Font::by_index({}));", name, v).unwrap();
            }
            if let Some(v) = &w.props.labelsize {
                writeln!(wid, "\t{}.set_label_size({});", name, v).unwrap();
            }
            if let Some(v) = &w.props.labelcolor {
                writeln!(wid, "\t{}.set_label_color(Color::by_index({}));", name, v).unwrap();
            }
        } else {
            sub.push(w.props.label.as_ref().unwrap_or(&String::new()).to_string());
            let name = &format!(
                "{}.find_item(\"{}\").unwrap()",
                *LAST_MENU.lock().unwrap(),
                {
                    let mut s = sub.iter().map(|x| x.to_owned() + "/").collect::<String>();
                    s.pop();
                    s
                }
            );
            if let Some(v) = &w.props.labeltype {
                let temp = utils::global_to_pascal(v);
                let temp = if temp == "No" { "None" } else { temp.as_str() };
                writeln!(substyle, "\t{}.set_label_type(LabelType::{});", name, temp).unwrap();
            }
            if let Some(v) = &w.props.labelfont {
                writeln!(
                    substyle,
                    "\t{}.set_label_font(Font::by_index({}));",
                    name, v
                )
                .unwrap();
            }
            if let Some(v) = &w.props.labelsize {
                writeln!(substyle, "\t{}.set_label_size({});", name, v).unwrap();
            }
            if let Some(v) = &w.props.labelcolor {
                writeln!(
                    substyle,
                    "\t{}.set_label_color(Color::by_index({}));",
                    name, v
                )
                .unwrap();
            }
        }
        if !w.children.is_empty() {
            wid += &add_menus(&w.children, sub);
        }
        if w.children.last().is_some() {
            sub.pop();
        }
    }
    wid += &substyle;
    wid
}

fn add_widgets(
    parent: Option<&str>,
    widgets: &[Widget],
    named: &mut Vec<(String, String)>,
) -> String {
    let mut wid = String::new();
    let mut flex = String::new();
    for w in widgets {
        let mut name = String::new();
        let mut refname = String::new();
        let typ = if let Some(class) = &w.props.class {
            class.to_owned()
        } else {
            utils::de_fl(&w.typ)
        };
        if typ != "MenuItem" && typ != "Submenu" {
            if let Some(comment) = &w.props.comment {
                wid += "\t// ";
                wid += comment;
                wid += "\n";
            }
            wid += "\tlet mut ";
            if w.name.is_empty() {
                let val = COUNTER.load(atomic::Ordering::Relaxed);
                name += "fl2rust_widget_";
                name += &val.to_string();
                COUNTER.store(val + 1, atomic::Ordering::Relaxed);
            } else {
                name += &w.name;
                named.push((name.clone(), typ.clone()));
            }
            if w.props.class.is_some() {
                refname += "*";
                refname += &name;
            } else {
                refname = name.clone();
            }
            wid += &name;
            wid += " = ";
            wid += &typ;
            wid += "::new(";
            for coord in w.props.xywh.split_ascii_whitespace() {
                wid += coord;
                wid += ", ";
            }
            wid += "None);\n";
            if let Some(label) = &w.props.label {
                wid += "\t";
                wid += &name;
                wid += ".set_label(";
                wid += &i18nize(label);
                wid += ");\n";
            }

            if is_parent_type(&typ) {
                wid += "\t";
                wid += &name;
                wid += ".end();\n";
            }

            if let Some(v) = &w.props.typ {
                let v = if typ == "Flex" {
                    if v == "HORIZONTAL" {
                        "Row"
                    } else {
                        "Column"
                    }
                } else {
                    v
                };
                writeln!(
                    wid,
                    "\t{}.set_type({}Type::{});",
                    name,
                    utils::fix_type(&typ),
                    utils::global_to_pascal(v)
                )
                .unwrap();
            } else if typ == "Flex" {
                writeln!(wid, "\t{}.set_type(FlexType::Column);", name,).unwrap();
            }
            if let Some(v) = &w.props.align {
                writeln!(
                    wid,
                    "\t{}.set_align(unsafe {{std::mem::transmute({})}});",
                    name, v
                )
                .unwrap();
            }
            if w.props.resizable.is_some() {
                if parent.is_none() {
                    writeln!(wid, "\t{}.make_resizable(true);", name).unwrap();
                } else {
                    writeln!(wid, "\t{}.resizable(&{});", parent.unwrap(), refname).unwrap();
                }
            }
            if w.props.visible.is_some() {
                writeln!(wid, "\t{}.show();", name).unwrap();
            }
            if w.props.hide.is_some() {
                writeln!(wid, "\t{}.hide();", name).unwrap();
            }
            if w.props.deactivate.is_some() {
                writeln!(wid, "\t{}.deactivate();", name).unwrap();
            }
            if let Some(v) = &w.props.color {
                writeln!(wid, "\t{}.set_color(Color::by_index({}));", name, v).unwrap();
            }
            if let Some(v) = &w.props.selection_color {
                writeln!(
                    wid,
                    "\t{}.set_selection_color(Color::by_index({}));",
                    name, v
                )
                .unwrap();
            }
            if let Some(v) = &w.props.tooltip {
                writeln!(wid, "\t{}.set_tooltip({});", name, i18nize(v)).unwrap();
            }
            if let Some(v) = &w.props.xclass {
                writeln!(wid, "\t{}.set_xclass({});", name, i18nize(v)).unwrap();
            }
            if w.props.noborder.is_some() {
                writeln!(wid, "\t{}.set_border(false);", name).unwrap();
            }
            if w.props.modal.is_some() {
                writeln!(wid, "\t{}.make_modal(true);", name).unwrap();
            }
            if w.props.non_modal.is_some() {
                writeln!(wid, "\t{}.make_modal(false);", name).unwrap();
            }
            if let Some(v) = &w.props.image {
                writeln!(wid, "\t{0}.set_image(Some(SharedImage::load(\"{1}\").expect(\"Could not find image: {1}\")));", name, v).unwrap();
            }
            if let Some(v) = &w.props.deimage {
                writeln!(wid, "\t{0}.set_deimage(Some(SharedImage::load(\"{1}\").expect(\"Could not find image: {1}\")));", name, v).unwrap();
            }
            if let Some(v) = &w.props.r#box {
                let temp = utils::global_to_pascal(v);
                let temp = match temp.as_str() {
                    "OflatBox" => "OFlatFrame",
                    "OshadowBox" => "OShadowBox",
                    "RflatBox" => "RFlatBox",
                    "RshadowBox" => "RShadowBox",
                    _ => temp.as_str(),
                };
                writeln!(wid, "\t{}.set_frame(FrameType::{});", name, temp).unwrap();
            }
            if let Some(v) = &w.props.down_box {
                let temp = utils::global_to_pascal(v);
                let temp = match temp.as_str() {
                    "OflatBox" => "OFlatFrame",
                    "OshadowBox" => "OShadowBox",
                    "RflatBox" => "RFlatBox",
                    "RshadowBox" => "RShadowBox",
                    _ => temp.as_str(),
                };
                writeln!(wid, "\t{}.set_down_frame(FrameType::{});", name, temp).unwrap();
            }
            if let Some(v) = &w.props.labeltype {
                let temp = utils::global_to_pascal(v);
                let temp = if temp == "No" { "None" } else { temp.as_str() };
                writeln!(wid, "\t{}.set_label_type(LabelType::{});", name, temp).unwrap();
            }
            if let Some(v) = &w.props.labelfont {
                writeln!(wid, "\t{}.set_label_font(Font::by_index({}));", name, v).unwrap();
            }
            if let Some(v) = &w.props.labelsize {
                writeln!(wid, "\t{}.set_label_size({});", name, v).unwrap();
            }
            if let Some(v) = &w.props.labelcolor {
                writeln!(wid, "\t{}.set_label_color(Color::by_index({}));", name, v).unwrap();
            }
            if let Some(v) = &w.props.when {
                writeln!(
                    wid,
                    "\t{}.set_trigger(unsafe {{std::mem::transmute({})}});",
                    name, v
                )
                .unwrap();
            }
            if let Some(v) = &w.props.textfont {
                writeln!(wid, "\t{}.set_text_font(Font::by_index({}));", name, v).unwrap();
            }
            if let Some(v) = &w.props.textsize {
                writeln!(wid, "\t{}.set_text_size({});", name, v).unwrap();
            }
            if let Some(v) = &w.props.textcolor {
                writeln!(wid, "\t{}.set_text_color(Color::by_index({}));", name, v).unwrap();
            }
            if let Some(v) = &w.props.shortcut {
                writeln!(
                    wid,
                    "\t{}.set_shortcut(unsafe {{std::mem::transmute({})}});",
                    name, v
                )
                .unwrap();
            }
            if let Some(v) = &w.props.gap {
                writeln!(wid, "\t{}.set_pad({});", name, v).unwrap();
            }
            if let Some(v) = &w.props.minimum {
                writeln!(wid, "\t{}.set_minimum({} as _);", name, v).unwrap();
            }
            if let Some(v) = &w.props.maximum {
                writeln!(wid, "\t{}.set_maximum({} as _);", name, v).unwrap();
            }
            if let Some(v) = &w.props.size {
                writeln!(wid, "\t{}.set_size({} as _);", name, v).unwrap();
            }
            if let Some(v) = &w.props.slider_size {
                writeln!(wid, "\t{}.set_slider_size({} as _);", name, v).unwrap();
            }
            if let Some(v) = &w.props.step {
                writeln!(wid, "\t{}.set_step({} as _, 1);", name, v).unwrap();
            }
            if let Some(v) = &w.props.user_data {
                if let Some(stripped) = v.strip_prefix("id:") {
                    writeln!(wid, "\t{}.set_id(\"{}\");", name, stripped).unwrap();
                }
            }
            if let Some(v) = &w.props.value {
                let val = if typ.contains("Button") {
                    let b = v
                        .parse::<i32>()
                        .expect("Buttons should have integral values");
                    if b != 0 {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                } else if (typ.contains("Input") || typ.contains("Output"))
                    && !typ.contains("Value")
                {
                    i18nize(v)
                } else {
                    format!("{} as _", v)
                };
                writeln!(wid, "\t{}.set_value({});", name, val).unwrap();
            }
            if let Some(v) = &w.props.code0 {
                wid += "\t";
                wid += v;
                wid += "\n";
            }
            if let Some(v) = &w.props.code1 {
                wid += "\t";
                wid += v;
                wid += "\n";
            }
            if let Some(v) = &w.props.code2 {
                wid += "\t";
                wid += v;
                wid += "\n";
            }
            if let Some(v) = &w.props.code3 {
                wid += "\t";
                wid += v;
                wid += "\n";
            }
            if let Some(v) = &w.props.extra_code {
                wid += "\t";
                wid += v;
                wid += "\n";
            }
            if let Some(v) = &w.props.callback {
                writeln!(wid, "\t{}.set_callback({});", name, v).unwrap();
            }

            if let Some(sizes) = &w.props.size_tuple {
                let count: Vec<_> = sizes.split_ascii_whitespace().collect();
                let count: Vec<_> = count.iter().skip(1).collect();
                for e in count.chunks_exact(2) {
                    let idx: usize = e[0].parse().unwrap();
                    writeln!(
                        flex,
                        "\t{0}.set_size(&{0}.child({1}).unwrap(), {2});",
                        name, idx, e[1]
                    )
                    .unwrap();
                }
                writeln!(flex, "\t{}.recalc();", name).unwrap();
            }

            if let Some(sizes) = &w.props.size_range {
                let count: Vec<_> = sizes.split_ascii_whitespace().collect();
                write!(wid, "\t{0}.size_range(", name).unwrap();
                for e in count {
                    wid += e;
                    wid += ", ";
                }
                wid += ");\n";
            }
            if let Some(parent) = parent {
                wid += "\t";
                wid += parent;
                wid += ".add(&";
                wid += &refname;
                wid += ");\n"
            }

            if is_menu_type(&typ) {
                {
                    *LAST_MENU.lock().unwrap() = name.to_string();
                }
                let ch = add_menus(&w.children, &mut vec![]);
                wid += &ch;
            } else if !w.children.is_empty() {
                let ch = add_widgets(Some(&name), &w.children, named);
                wid += &ch;
            }
        }
    }
    wid += &flex;
    wid
}

fn add_funcs(functions: &[Function], free: bool, named: &mut Vec<(String, String)>) -> String {
    let mut func = String::new();
    for c in functions {
        func += "\n    pub fn ";
        func += &c.name;
        if let Some(ret) = &c.props.return_type {
            func += " -> ";
            func += ret;
        } else if !free {
            func += " -> Self";
        }
        func += " {\n";
        if !c.widgets.is_empty() {
            func += &add_widgets(None, &c.widgets, named);
        }
        if free {
            func += "\t(\n";
        } else {
            func += "\tSelf {\n";
        }
        if !named.is_empty() && named.len() > 1 {
            for n in named.iter() {
                func += "\t    ";
                func += &n.0;
                func += ",\n";
            }
        } else if !named.is_empty() && named.len() == 1 {
            func += "\t    ";
            func += &named[0].0;
            func += "\n";
        }
        if free {
            func += "\t)";
        } else {
            func += "\t}";
        }
        func += "\n    }";
    }
    func
}

fn add_widget_class_ctor(w: &Widget, named: &mut Vec<(String, String)>) -> String {
    let mut wid = String::new();
    wid += "\n    pub fn new<L: Into<Option<&'static str>>>(x: i32, y: i32, w: i32, h: i32, label: L) -> Self {\n";
    wid += "\tlet mut base_group = Group::new(0, 0, ";
    for coord in w.props.xywh.split_ascii_whitespace().skip(2) {
        wid += coord;
        wid += ", ";
    }
    wid += "label);\n";
    let name = "base_group";
    if w.props.resizable.is_some() {
        writeln!(wid, "\t{}.make_resizable(true);", name).unwrap();
    }
    if let Some(v) = &w.props.labeltype {
        let temp = utils::global_to_pascal(v);
        let temp = if temp == "No" { "None" } else { temp.as_str() };
        writeln!(wid, "\t{}.set_label_type(LabelType::{});", name, temp).unwrap();
    }
    if let Some(v) = &w.props.labelfont {
        writeln!(wid, "\t{}.set_label_font(Font::by_index({}));", name, v).unwrap();
    }
    if let Some(v) = &w.props.labelsize {
        writeln!(wid, "\t{}.set_label_size({});", name, v).unwrap();
    }
    if let Some(v) = &w.props.labelcolor {
        writeln!(wid, "\t{}.set_label_color(Color::by_index({}));", name, v).unwrap();
    }
    if let Some(v) = &w.props.color {
        writeln!(wid, "\t{}.set_color(Color::by_index({}));", name, v).unwrap();
    }
    wid += "\tbase_group.end();\n";
    if !w.children.is_empty() {
        wid += &add_widgets(Some(name), &w.children, named);
    }
    wid += "\tbase_group.resize(x, y, w, h);\n";
    wid += "\tSelf {\n\t    base_group,\n";
    if !named.is_empty() {
        for n in named.iter() {
            wid += "\t    ";
            wid += &n.0;
            wid += ",\n";
        }
    }
    wid += "\t}";
    wid += "\n    }";
    wid
}

/// Generate the output Rust string/file
fn generate_(ast: &Ast) -> String {
    let mut s = String::new();
    if let Some(i18n) = ast.i18n_type {
        I18N.store(i18n, atomic::Ordering::Relaxed);
    }
    s += "\n";
    let mut classes = vec![];
    let mut widget_classes = vec![];
    let mut funcs = vec![];
    if !ast.decls.is_empty() {
        for decl in &ast.decls {
            s += &decl.decl;
            s += "\n";
        }
        s += "\n";
    }
    if !ast.comments.is_empty() {
        for comment in &ast.comments {
            s += &comment.comment;
            s += "\n";
        }
    }
    if !ast.functions.is_empty() {
        let mut local_named = vec![];
        let func = add_funcs(&ast.functions, true, &mut local_named);
        funcs.push(func);
    }
    if !ast.widget_classes.is_empty() {
        let mut named: Vec<(String, String)> = vec![];
        let mut class = String::new();
        for c in &ast.widget_classes {
            class += "#[derive(Debug, Clone)]\n";
            class += "pub struct ";
            class += &c.name;
            class += " {\n";
            class += "    pub base_group: Group,\n";
            let fns = add_widget_class_ctor(c, &mut named);
            if !named.is_empty() {
                for n in &named {
                    class += "    pub ";
                    class += &n.0;
                    class += ": ";
                    class += &n.1;
                    class += ",\n";
                }
            }
            named.clear();
            class += "}\n\n";
            class += "impl ";
            class += &c.name;
            class += " {";
            class += &fns;
            class += "\n}\n\n";
            class += "fltk::widget_extends!(";
            class += &c.name;
            class += ", Group, base_group);\n\n";
        }
        widget_classes.push(class);
    }
    if !ast.classes.is_empty() {
        let mut named: Vec<(String, String)> = vec![];
        let mut class = String::new();
        for c in &ast.classes {
            class += "#[derive(Debug, Clone)]\n";
            class += "pub struct ";
            class += &c.name;
            class += " {\n";
            let fns = add_funcs(&c.functions, false, &mut named);
            if !named.is_empty() {
                for n in &named {
                    class += "    pub ";
                    class += &n.0;
                    class += ": ";
                    class += &n.1;
                    class += ",\n";
                }
            }
            named.clear();
            class += "}\n\n";
            if !c.functions.is_empty() {
                class += "impl ";
                class += &c.name;
                class += " {";
                class += &fns;
                class += "\n}\n\n";
            }
        }
        classes.push(class);
    }
    for f in funcs {
        s += &f;
        s += "\n";
    }
    for c in widget_classes {
        s += &c;
        s += "\n";
    }
    for c in classes {
        s += &c;
        s += "\n";
    }
    s
}

/// Generate the output Rust string/file
pub fn generate(ast: &Ast) -> String {
    let s = generate_(ast);
    format!("{}\n{}", HEADER, s)
}

/// Generate the output Rust string/file
pub fn generate_with_directives_preamble(ast: &Ast) -> String {
    let s = generate_(ast);
    format!("{}\n{}\n{}", ALLOWS, HEADER, s)
}
