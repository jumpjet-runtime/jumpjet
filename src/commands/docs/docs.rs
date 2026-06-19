use std::{
    collections::{HashMap, VecDeque},
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_widget_list::{ListBuilder, ListState, ListView};
use wit_parser::{
    Function, FunctionKind, Interface, InterfaceId, SourceMap, Type, TypeDef, TypeId,
    UnresolvedPackage, UnresolvedPackageGroup,
};

use crate::{
    action::Action,
    assets::JumpjetRuntimeWits,
    components::Component,
    config::{Config, KeyBindings},
    tui::Event,
};

#[derive(Clone)]
pub enum CurrentItem {
    Package(UnresolvedPackage),
    Interface(InterfaceId),
    TypeDef(InterfaceId, TypeId),
    Function(Function),
}

impl CurrentItem {
    pub fn render<'a>(&self, package: &UnresolvedPackage) -> Line<'a> {
        match self {
            CurrentItem::Package(p) => Line::from(p.name.name.clone()),
            CurrentItem::Interface(id) => {
                let interface = package.interfaces.get(*id).unwrap();
                Line::from(interface.name.clone().unwrap_or_default())
            }
            CurrentItem::TypeDef(interface_id, type_id) => {
                let type_def = package.types.get(*type_id).unwrap();
                Line::from(type_def_name(package, type_def))
            }
            CurrentItem::Function(func) => render_function_signature(package, func),
        }
    }
}

fn list_items<'a>(
    package: &UnresolvedPackage,
    item: &CurrentItem,
) -> Vec<(Line<'a>, Line<'a>, CurrentItem)> {
    match item {
        CurrentItem::Package(package) => package
            .interfaces
            .iter()
            .map(|(interface_id, _)| render_interface_list_item(package, &interface_id))
            .collect(),
        CurrentItem::Interface(interface_id) => {
            let interface = package.interfaces.get(*interface_id).unwrap();
            let mut functions = interface
                .functions
                .iter()
                .filter(|(_, f)| f.kind == FunctionKind::Freestanding)
                .map(|(_, f)| render_function_list_item(package, f))
                .collect::<Vec<_>>();

            let types = interface
                .types
                .iter()
                .map(|(_, type_id)| render_type_list_item(package, interface_id, type_id))
                .collect::<Vec<_>>();

            functions.extend(types);

            functions
        }
        CurrentItem::TypeDef(interface_id, type_id) => {
            let interface = package.interfaces.get(*interface_id).unwrap();
            interface
                .functions
                .iter()
                .filter(|(_, f)| match f.kind {
                    FunctionKind::Method(tid) => tid.eq(type_id),
                    FunctionKind::Static(tid) => tid.eq(type_id),
                    FunctionKind::Constructor(tid) => tid.eq(type_id),
                    _ => false,
                })
                .map(|(_, f)| render_function_list_item(package, f))
                .collect::<Vec<_>>()
        }
        // A function is a leaf in the docs tree; it has no child items to list.
        CurrentItem::Function(_) => Vec::new(),
    }
}

fn type_def_name(package: &UnresolvedPackage, type_def: &TypeDef) -> String {
    let type_def_kind = &type_def.kind;

    match type_def_kind {
        wit_parser::TypeDefKind::Record(record) => type_def.name.as_ref().unwrap().clone(),
        wit_parser::TypeDefKind::Resource => type_def.name.as_ref().unwrap().clone(),
        wit_parser::TypeDefKind::Handle(handle) => match handle {
            wit_parser::Handle::Own(id) | wit_parser::Handle::Borrow(id) => {
                let type_def = package.types.get(*id).unwrap();
                type_def.name.as_ref().unwrap().clone()
            }
        },
        wit_parser::TypeDefKind::Flags(flags) => flags
            .flags
            .iter()
            .map(|f| f.name.clone())
            .collect::<Vec<_>>()
            .join(" | ")
            .to_owned(),
        wit_parser::TypeDefKind::Tuple(tuple) => {
            let tuple = tuple
                .types
                .iter()
                .map(|t| type_name(package, t))
                .collect::<Vec<_>>()
                .join(", ");

            format!("({tuple})")
        }
        wit_parser::TypeDefKind::Variant(variant) => type_def.name.as_ref().unwrap().clone(),
        wit_parser::TypeDefKind::Enum(_) => type_def.name.as_ref().unwrap().clone(),
        wit_parser::TypeDefKind::Option(t) => format!("option<{}>", type_name(package, t)),
        wit_parser::TypeDefKind::Result(result) => format!(
            "result<{}, {}>",
            match result.ok {
                Some(t) => type_name(package, &t),
                None => "()".to_owned(),
            },
            match result.err {
                Some(t) => type_name(package, &t),
                None => "()".to_owned(),
            }
        ),
        wit_parser::TypeDefKind::List(t) => format!("list<{}>", type_name(package, &t)),
        wit_parser::TypeDefKind::Future(t) => match t {
            Some(t) => format!("future<{}>", type_name(package, &t)),
            None => "future".to_owned(),
        },
        wit_parser::TypeDefKind::Map(k, v) => {
            format!("map<{}: {}>", type_name(package, k), type_name(package, v))
        }
        wit_parser::TypeDefKind::FixedLengthList(t, n) => {
            format!("list<{}, {}>", type_name(package, t), n)
        }
        wit_parser::TypeDefKind::Type(t) => type_name(package, &t),
        wit_parser::TypeDefKind::Unknown => "?".to_owned(),
        _ => "unknown".to_owned(),
    }
}

fn type_name(package: &UnresolvedPackage, type_: &Type) -> String {
    match type_ {
        Type::Bool => "bool".to_owned(),
        Type::U8 => "u8".to_owned(),
        Type::U16 => "u16".to_owned(),
        Type::U32 => "u32".to_owned(),
        Type::U64 => "u64".to_owned(),
        Type::S8 => "s8".to_owned(),
        Type::S16 => "s16".to_owned(),
        Type::S32 => "s32".to_owned(),
        Type::S64 => "s64".to_owned(),
        Type::F32 => "f32".to_owned(),
        Type::F64 => "f64".to_owned(),
        Type::Char => "char".to_owned(),
        Type::String => "string".to_owned(),
        Type::ErrorContext => "error-context".to_owned(),
        Type::Id(id) => type_def_name(package, package.types.get(*id).unwrap()),
    }
}

fn render_type<'a>(package: &UnresolvedPackage, type_: &Type) -> Span<'a> {
    match type_ {
        Type::Id(id) => Span::styled(
            type_def_name(package, package.types.get(*id).unwrap()),
            Style::reset().bold().fg(Color::Indexed(43)),
        ),
        Type::Bool => Span::styled("bool", Style::reset().bold().fg(Color::Indexed(75))),
        Type::U8 => Span::styled("u8", Style::reset().bold().fg(Color::Indexed(75))),
        Type::U16 => Span::styled("u16", Style::reset().bold().fg(Color::Indexed(75))),
        Type::U32 => Span::styled("u32", Style::reset().bold().fg(Color::Indexed(75))),
        Type::U64 => Span::styled("u64", Style::reset().bold().fg(Color::Indexed(75))),
        Type::S8 => Span::styled("s8", Style::reset().bold().fg(Color::Indexed(75))),
        Type::S16 => Span::styled("s16", Style::reset().bold().fg(Color::Indexed(75))),
        Type::S32 => Span::styled("s32", Style::reset().bold().fg(Color::Indexed(75))),
        Type::S64 => Span::styled("s64", Style::reset().bold().fg(Color::Indexed(75))),
        Type::F32 => Span::styled("f32", Style::reset().bold().fg(Color::Indexed(75))),
        Type::F64 => Span::styled("f64", Style::reset().bold().fg(Color::Indexed(75))),
        Type::Char => Span::styled("char", Style::reset().bold().fg(Color::Indexed(75))),
        Type::String => Span::styled("string", Style::reset().bold().fg(Color::Indexed(75))),
        Type::ErrorContext => Span::styled(
            "error-context",
            Style::reset().bold().fg(Color::Indexed(75)),
        ),
    }
}

fn render_function_signature<'a>(package: &UnresolvedPackage, func: &Function) -> Line<'a> {
    match func.kind {
        wit_parser::FunctionKind::Freestanding | wit_parser::FunctionKind::AsyncFreestanding => {
            let name = func.item_name();
            let mut rendered_params = func
                .params
                .iter()
                .skip(1)
                .map(|param| {
                    vec![
                        Span::styled(
                            format!("{}: ", param.name),
                            Style::reset().bold().fg(Color::White),
                        ),
                        render_type(&package, &param.ty),
                        Span::styled(format!(", "), Style::reset().bold().fg(Color::White)),
                    ]
                })
                .flat_map(|x| x)
                .collect::<Vec<_>>();

            rendered_params.pop();

            let rendered_return = func
                .result
                .iter()
                .map(|t| render_type(package, t))
                .collect::<Vec<_>>();

            let mut rendered_func = vec![];
            rendered_func.extend([
                Span::styled("static fn ", Style::reset().bold().fg(Color::Indexed(33))),
                Span::styled(
                    format!("{name}"),
                    Style::reset().bold().fg(Color::Indexed(229)),
                ),
                Span::styled(format!("("), Style::reset().bold().fg(Color::Indexed(229))),
            ]);
            rendered_func.extend(rendered_params);
            rendered_func.extend([Span::styled(
                format!(")"),
                Style::reset().bold().fg(Color::Indexed(229)),
            )]);
            if rendered_return.len() > 0 {
                rendered_func.push(Span::styled(format!(": "), Style::reset().bold().dim()));
                rendered_func.extend(rendered_return);
            }

            Line::from(rendered_func)
        }
        wit_parser::FunctionKind::Method(type_id)
        | wit_parser::FunctionKind::AsyncMethod(type_id) => {
            let name = func.item_name();
            let mut rendered_params = func
                .params
                .iter()
                .skip(1)
                .map(|param| {
                    vec![
                        Span::styled(
                            format!("{}: ", param.name),
                            Style::reset().bold().fg(Color::White),
                        ),
                        render_type(&package, &param.ty),
                        Span::styled(format!(", "), Style::reset().bold().fg(Color::White)),
                    ]
                })
                .flat_map(|x| x)
                .collect::<Vec<_>>();

            rendered_params.pop();

            let rendered_return = func
                .result
                .iter()
                .map(|t| render_type(package, t))
                .collect::<Vec<_>>();

            let mut rendered_func = vec![];
            rendered_func.extend([
                Span::styled("fn ", Style::reset().bold().fg(Color::Indexed(33))),
                Span::styled(
                    format!("{name}"),
                    Style::reset().bold().fg(Color::Indexed(229)),
                ),
                Span::styled(format!("("), Style::reset().bold().fg(Color::Indexed(229))),
            ]);
            rendered_func.extend(rendered_params);
            rendered_func.extend([Span::styled(
                format!(")"),
                Style::reset().bold().fg(Color::Indexed(229)),
            )]);
            if rendered_return.len() > 0 {
                rendered_func.push(Span::styled(format!(": "), Style::reset().bold().dim()));
                rendered_func.extend(rendered_return);
            }

            Line::from(rendered_func)
        }
        wit_parser::FunctionKind::Static(type_id)
        | wit_parser::FunctionKind::AsyncStatic(type_id) => {
            let name = func.item_name();
            let mut rendered_params = func
                .params
                .iter()
                .map(|param| {
                    vec![
                        Span::styled(
                            format!("{}: ", param.name),
                            Style::reset().bold().fg(Color::White),
                        ),
                        render_type(&package, &param.ty),
                        Span::styled(format!(", "), Style::reset().bold().fg(Color::White)),
                    ]
                })
                .flat_map(|x| x)
                .collect::<Vec<_>>();

            rendered_params.pop();

            let rendered_return = func
                .result
                .iter()
                .map(|t| render_type(package, t))
                .collect::<Vec<_>>();

            let mut rendered_func = vec![];
            rendered_func.extend([
                Span::styled("static fn ", Style::reset().bold().fg(Color::Indexed(33))),
                Span::styled(
                    format!("{name}"),
                    Style::reset().bold().fg(Color::Indexed(229)),
                ),
                Span::styled(format!("("), Style::reset().bold().fg(Color::Indexed(229))),
            ]);
            rendered_func.extend(rendered_params);
            rendered_func.extend([Span::styled(
                format!(")"),
                Style::reset().bold().fg(Color::Indexed(229)),
            )]);
            if rendered_return.len() > 0 {
                rendered_func.push(Span::styled(format!(": "), Style::reset().bold().dim()));
                rendered_func.extend(rendered_return);
            }

            Line::from(rendered_func)
        }
        wit_parser::FunctionKind::Constructor(type_id) => {
            let type_def = package.types.get(type_id).unwrap();
            let name = type_def.name.clone().unwrap();
            let mut rendered_params = func
                .params
                .iter()
                .map(|param| {
                    vec![
                        Span::styled(
                            format!("{}: ", param.name),
                            Style::reset().bold().fg(Color::White),
                        ),
                        render_type(&package, &param.ty),
                        Span::styled(format!(", "), Style::reset().bold().fg(Color::White)),
                    ]
                })
                .flat_map(|x| x)
                .collect::<Vec<_>>();

            rendered_params.pop();

            let rendered_return = func
                .result
                .iter()
                .map(|t| render_type(package, t))
                .collect::<Vec<_>>();

            let mut rendered_func = vec![];
            rendered_func.extend([
                Span::styled("constructor ", Style::reset().bold().fg(Color::Indexed(33))),
                Span::styled(
                    format!("{name}"),
                    Style::reset().bold().fg(Color::Indexed(43)),
                ),
                Span::styled(format!("("), Style::reset().bold().fg(Color::Indexed(43))),
            ]);
            rendered_func.extend(rendered_params);
            rendered_func.extend([Span::styled(
                format!(")"),
                Style::reset().bold().fg(Color::Indexed(43)),
            )]);
            if rendered_return.len() > 0 {
                rendered_func.push(Span::styled(format!(": "), Style::reset().bold().dim()));
                rendered_func.extend(rendered_return);
            }

            Line::from(rendered_func)
        }
    }
}

fn render_interface_list_item<'a>(
    package: &UnresolvedPackage,
    interface_id: &InterfaceId,
) -> (Line<'a>, Line<'a>, CurrentItem) {
    let interface = package.interfaces.get(*interface_id).unwrap();
    (
        Line::from(vec![
            Span::styled("namespace ", Style::reset().bold().fg(Color::Indexed(33))),
            Span::styled(
                interface.name.clone().unwrap_or_default(),
                Style::reset().bold().fg(Color::Indexed(75)),
            ),
        ]),
        Line::styled(
            interface.docs.contents.clone().unwrap_or_default(),
            Style::reset().dim(),
        ),
        CurrentItem::Interface(*interface_id),
    )
}

fn render_type_list_item<'a>(
    package: &UnresolvedPackage,
    interface_id: &InterfaceId,
    type_id: &TypeId,
) -> (Line<'a>, Line<'a>, CurrentItem) {
    let interface = package.interfaces.get(*interface_id).unwrap();
    let type_def = package.types.get(*type_id).unwrap();
    let kind_str = match &type_def.kind {
        wit_parser::TypeDefKind::Record(record) => "record",
        wit_parser::TypeDefKind::Resource => "resource",
        wit_parser::TypeDefKind::Handle(handle) => "handle",
        wit_parser::TypeDefKind::Flags(flags) => "flags",
        wit_parser::TypeDefKind::Tuple(tuple) => "tuple",
        wit_parser::TypeDefKind::Variant(variant) => "variant",
        wit_parser::TypeDefKind::Enum(_) => "enum",
        wit_parser::TypeDefKind::Option(t) => &format!("option<{}>", type_name(package, t)),
        wit_parser::TypeDefKind::Result(result) => "result",
        wit_parser::TypeDefKind::List(t) => &format!("list<{}>", type_name(package, t)),
        wit_parser::TypeDefKind::Future(_) => "future",
        wit_parser::TypeDefKind::Stream(_) => "stream",
        wit_parser::TypeDefKind::Map(_, _) => "map",
        wit_parser::TypeDefKind::FixedLengthList(_, _) => "fixed-length-list",
        _ => "type",
    };
    (
        Line::from(vec![
            Span::styled(
                format!("{kind_str} "),
                Style::reset().bold().fg(Color::Indexed(43)),
            ),
            Span::styled(
                type_def.name.clone().unwrap_or_default(),
                Style::reset().bold().fg(Color::Indexed(158)),
            ),
        ]),
        Line::styled(
            type_def.docs.contents.clone().unwrap_or_default(),
            Style::reset().dim(),
        ),
        CurrentItem::TypeDef(*interface_id, *type_id),
    )
}

fn render_function_list_item<'a>(
    package: &UnresolvedPackage,
    func: &Function,
) -> (Line<'a>, Line<'a>, CurrentItem) {
    let item = CurrentItem::Function(func.clone());

    (
        render_function_signature(package, func),
        Line::styled(
            func.docs.contents.clone().unwrap_or_default(),
            Style::reset().dim(),
        ),
        item,
    )
}

fn render_function_page(
    package: &UnresolvedPackage,
    func: &Function,
    f: &mut Frame,
    area: Rect,
    function_params_list_state: &mut ListState,
) {
    let layout = Layout::vertical(vec![Constraint::Min(0), Constraint::Min(0)]);

    // let params_area_layout = Layout::vertical(vec![
    //     Constraint::Min(0),
    //     Constraint::Fill(1),
    // ]);

    let [description_area, content_area] = layout.areas(area);
    // let [params_header_area, params_list_area] = params_area_layout.areas(content_area);

    let name = func.item_name();
    let params = func
        .params
        .iter()
        .skip(1)
        .map(|param| format!("{}", param.name))
        .collect::<Vec<_>>()
        .join(", ");

    let description = Paragraph::new(func.docs.contents.clone().unwrap_or_default())
        .block(Block::new().padding(Padding::proportional(1)));

    let params_list: Vec<_> = func
        .params
        .clone()
        .into_iter()
        .filter(|p| p.name != "self")
        .collect();
    let params_count = params_list.len();

    let params_header =
        Paragraph::new("Parameters").block(Block::new().padding(Padding::proportional(1)));

    let package = package.clone();
    let params_list_builder = ListBuilder::new(move |context| {
        let param = &params_list[context.index];
        let item = Paragraph::new(vec![Line::from(vec![
            Span::from(format!("{}: ", param.name)),
            render_type(&package, &param.ty),
        ])])
        .block(
            Block::new()
                .padding(Padding::horizontal(1))
                .borders(Borders::LEFT | Borders::BOTTOM)
                .border_set(if context.is_selected {
                    symbols::border::Set {
                        top_left: symbols::line::THICK.vertical,
                        top_right: " ",
                        bottom_left: " ",
                        bottom_right: " ",
                        vertical_left: symbols::line::THICK.vertical,
                        vertical_right: " ",
                        horizontal_top: " ",
                        horizontal_bottom: " ",
                    }
                } else {
                    symbols::border::EMPTY
                })
                .border_style(if context.is_selected {
                    Style::reset()
                } else {
                    Style::default()
                }),
        );

        (item, 3)
    });

    let params_list = ListView::new(params_list_builder, params_count)
        .block(Block::default().padding(Padding::vertical(1)));

    f.render_widget(description, description_area);

    // f.render_widget(params_header, params_header_area);
    // f.render_stateful_widget(params_list, params_list_area, function_params_list_state);
}

pub struct Docs {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    visited: VecDeque<CurrentItem>,
    list_state: ListState,
    function_params_list_state: ListState,
}

impl Docs {
    fn package(&self) -> &UnresolvedPackage {
        if let CurrentItem::Package(p) = self.visited.front().unwrap() {
            p
        } else {
            panic!("Not a package");
        }
    }
}

impl Default for Docs {
    fn default() -> Self {
        let mut source_map = SourceMap::new();
        for wit_name in JumpjetRuntimeWits::iter() {
            let wit = JumpjetRuntimeWits::get(&wit_name).unwrap();
            source_map.push(
                Path::new(wit_name.as_ref()),
                std::str::from_utf8(&wit.data).unwrap(),
            );
        }
        let package_group = source_map.parse().unwrap();

        let mut visited = VecDeque::new();
        visited.push_back(CurrentItem::Package(package_group.main));

        Self {
            command_tx: None,
            config: Config::default(),
            visited,
            list_state: ListState::default(),
            function_params_list_state: ListState::default(),
        }
    }
}

impl Component for Docs {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            _ => {}
        }
        Ok(None)
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key {
            KeyEvent {
                code: KeyCode::Up, ..
            } => self.list_state.previous(),
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => self.list_state.next(),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                if let Some(selected) = self.list_state.selected {
                    if let Some(back) = self.visited.back() {
                        let items: Vec<_> =
                            list_items(self.package(), &self.visited.back().unwrap());
                        let (_, _, selected_item) = &items[selected];
                        self.visited.push_back(selected_item.clone());
                        self.list_state.select(None);
                    }
                }
            }
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                if self.visited.len() > 1 {
                    self.visited.pop_back();
                    self.list_state.select(None);
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [breadcrumb_area, content_area] = layout.areas(area);

        let breadcrumbs = self
            .visited
            .iter()
            .map(|x| x.render(self.package()))
            .collect::<Vec<_>>();
        let breadcrumb_depth = breadcrumbs.len() - 1;

        f.render_widget(
            Tabs::new(breadcrumbs)
                .block(Block::default().padding(Padding::horizontal(1)))
                .style(Style::reset().dim())
                .highlight_style(Style::new().bg(Color::Reset))
                .select(breadcrumb_depth)
                .divider("→"),
            breadcrumb_area,
        );

        let package = self.package().clone();

        match self.visited.back().unwrap() {
            CurrentItem::Function(func) => {
                let layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]);

                render_function_page(
                    &package,
                    func,
                    f,
                    content_area,
                    &mut self.function_params_list_state,
                );
            }
            _ => {
                let items: Vec<_> = list_items(self.package(), &self.visited.back().unwrap());
                let item_count = items.len();

                let builder = ListBuilder::new(move |context| {
                    let (mut name, mut description, _) = items[context.index].clone();
                    name.spans.insert(0, Span::from(" "));
                    description.spans.insert(0, Span::from(" "));

                    let item = Paragraph::new(vec![name, description]).block(
                        Block::new()
                            .borders(Borders::LEFT | Borders::BOTTOM)
                            .border_set(if context.is_selected {
                                symbols::border::Set {
                                    top_left: symbols::line::THICK.vertical,
                                    top_right: " ",
                                    bottom_left: " ",
                                    bottom_right: " ",
                                    vertical_left: symbols::line::THICK.vertical,
                                    vertical_right: " ",
                                    horizontal_top: " ",
                                    horizontal_bottom: " ",
                                }
                            } else {
                                symbols::border::EMPTY
                            })
                            .border_style(if context.is_selected {
                                Style::reset()
                            } else {
                                Style::default()
                            }),
                    );

                    (item, 3)
                });

                let list = ListView::new(builder, item_count)
                    .block(Block::default().padding(Padding::vertical(1)));

                f.render_stateful_widget(list, content_area, &mut self.list_state);
            }
        }

        Ok(())
    }
}
