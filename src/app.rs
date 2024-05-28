// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::fl;
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::Column;
use cosmic::widget::{self, menu};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Apply, Element};
use rustemon::model::pokemon::{Pokemon, PokemonStat, PokemonType};
use rustemon::model::resource::NamedApiResource;

const REPOSITORY: &str = "https://github.com/mariinkys/cosmicdex";

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct CosmicDex {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Currently selected Page
    current_page: Page,
    /// The rustemon client for interacting with PokeApi
    //rustemon_client: rustemon::client::RustemonClient,
    /// Contains the list of all Pokémon
    pokemon_list: Vec<NamedApiResource<Pokemon>>,
    /// Currently viewing Pokémon
    selected_pokemon: Option<Pokemon>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    LoadedPokemonList(Vec<NamedApiResource<Pokemon>>),
    LoadPokemon(String),
    LoadedPokemon(Pokemon),
    ReturnToLandingPage,
}

/// Identifies a page in the application.
pub enum Page {
    LandingPage,
    PokemonPage,
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Back,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Back => Message::ReturnToLandingPage,
        }
    }
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for CosmicDex {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.mariinkys.CosmicDex";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut app = CosmicDex {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            current_page: Page::LandingPage,
            //rustemon_client: rustemon::client::RustemonClient::default(),
            pokemon_list: Vec::<NamedApiResource<Pokemon>>::new(),
            selected_pokemon: None,
        };

        let cmd = cosmic::app::Command::perform(load_all_pokemon(), |pokemon_list| {
            cosmic::app::message::app(Message::LoadedPokemonList(pokemon_list))
        });
        let commands = Command::batch(vec![app.update_titles(), cmd]);

        (app, commands)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![
            menu::Tree::with_children(
                menu::root(fl!("view")),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(fl!("about"), MenuAction::About)],
                ),
            ),
            //TODO: This should be a button that allows to go back?
            menu::Tree::with_children(
                menu::root(fl!("back")),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(fl!("back"), MenuAction::Back)],
                ),
            ),
        ]);

        vec![menu_bar.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::LandingPage => self.landing(),
            Page::PokemonPage => self.pokemon_page(),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LaunchUrl(url) => {
                let _result = open::that_detached(url);
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                // Set the title of the context drawer.
                self.set_context_title(context_page.title());
            }
            Message::LoadedPokemonList(pokemons) => {
                self.pokemon_list = pokemons;
            }
            Message::LoadedPokemon(pokemon) => {
                self.selected_pokemon = Some(pokemon);
                self.current_page = Page::PokemonPage;
                let _ = self.update_titles();
            }
            Message::LoadPokemon(pokemon_name) => {
                return cosmic::app::Command::perform(load_pokemon(pokemon_name), |pokemon| {
                    cosmic::app::message::app(Message::LoadedPokemon(pokemon))
                });
            }
            Message::ReturnToLandingPage => self.current_page = Page::LandingPage,
        }
        Command::none()
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
        })
    }
}

impl CosmicDex {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(
            &include_bytes!("../res/icons/hicolor/128x128/apps/dev.mariinkys.CosmicDex.svg")[..],
        ));

        let title = widget::text::title3(fl!("app-title"));

        let app_info = widget::text::text(fl!("app_info"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(app_info)
            .push(link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn landing(&self) -> Element<Message> {
        let space_xxs = theme::active().cosmic().spacing.space_xxs;

        let children = self.pokemon_list.iter().map(|pokemon| {
            widget::button(
                widget::text::text(pokemon.name.to_string())
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .horizontal_alignment(Horizontal::Center),
            )
            .on_press_down(Message::LoadPokemon(pokemon.name.to_string()))
            .into()
        });

        widget::scrollable(
            Column::with_children(children)
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .spacing(space_xxs),
        )
        .into()
    }

    // pub fn testing_error_page(&self) -> Element<Message> {
    //     widget::text::title1(fl!("generic_error"))
    //         .apply(widget::container)
    //         .width(Length::Fill)
    //         .height(Length::Fill)
    //         .align_x(Horizontal::Center)
    //         .align_y(Vertical::Center)
    //         .into()
    // }

    pub fn pokemon_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let content: widget::Column<_> = match &self.selected_pokemon {
            Some(pokemon) => {
                let page_title = widget::text::title1(capitalize_string(pokemon.name.as_str()))
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Center);

                // TODO: Load the pokemon image having the URL
                // let pokemon_image_url = pokemon.sprites.front_default.clone().unwrap_or_default();
                // let pokemon_image = widget::Image::new(pokemon_image_url)
                //     .content_fit(cosmic::iced::ContentFit::Fill);
                let pokemon_image = widget::Image::new("tmp/ditto_front.png")
                    .content_fit(cosmic::iced::ContentFit::Fill);

                let pokemon_weight = widget::container::Container::new(
                    widget::Column::new()
                        .push(widget::text::title3("WEIGHT"))
                        .push(
                            widget::text::text(format!(
                                "{} Kg",
                                scale_numbers(pokemon.weight).to_string()
                            ))
                            .size(15.0),
                        )
                        .align_items(Alignment::Center),
                )
                .style(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_height = widget::container::Container::new(
                    widget::Column::new()
                        .push(widget::text::title3("HEIGHT"))
                        .push(
                            widget::text::text(format!(
                                "{} m",
                                scale_numbers(pokemon.height).to_string()
                            ))
                            .size(15.0),
                        )
                        .align_items(Alignment::Center),
                )
                .style(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let parsed_pokemon_types = self.parse_pokemon_types(&pokemon.types, &spacing);

                let pokemon_first_row = widget::Row::new()
                    .push(pokemon_weight)
                    .push(pokemon_height)
                    .push(parsed_pokemon_types)
                    .spacing(8.0)
                    .align_items(Alignment::Center);

                let parsed_pokemon_stats = self.parse_pokemon_stats(&pokemon.stats, &spacing);

                widget::Column::new()
                    .push(page_title)
                    .push(pokemon_image)
                    .push(pokemon_first_row)
                    .push(parsed_pokemon_stats)
                    .align_items(Alignment::Center)
                    .into()
            }
            None => {
                let error = widget::text::title1(fl!("generic_error"))
                    .apply(widget::container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center);

                widget::Column::new().push(error).into()
            }
        };

        widget::container(content).into()
    }

    /// Updates the header and window titles.
    pub fn update_titles(&mut self) -> Command<Message> {
        let mut window_title = fl!("app-title");
        let mut header_title = String::new();

        match self.current_page {
            Page::LandingPage => {
                window_title.push_str(" — ");
                window_title.push_str("All Pokémon");
                header_title.push_str("All Pokémon");
            }
            Page::PokemonPage => {
                window_title.push_str(" — ");
                window_title.push_str("Pokémon");
                header_title.push_str("Pokémon");
            }
        }

        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }

    pub fn parse_pokemon_stats(
        &self,
        stats: &Vec<PokemonStat>,
        spacing: &cosmic_theme::Spacing,
    ) -> Element<Message> {
        //TODO: Missing card title

        let children = stats.iter().map(|pokemon_stats| {
            widget::Row::new()
                .push(widget::text(capitalize_string(&pokemon_stats.stat.name)))
                .push(widget::text(pokemon_stats.base_stat.to_string()))
                .spacing(10.0)
                .into()
        });

        widget::container::Container::new(
            Column::with_children(children).align_items(Alignment::Center),
        )
        .style(theme::Container::ContextDrawer)
        .padding([spacing.space_none, spacing.space_xxs])
        .into()
    }

    pub fn parse_pokemon_types(
        &self,
        types: &Vec<PokemonType>,
        spacing: &cosmic_theme::Spacing,
    ) -> Element<Message> {
        //TODO: Missing card title

        let children = types.iter().map(|pokemon_types| {
            widget::Row::new()
                .push(widget::text(pokemon_types.type_.name.to_uppercase()))
                .into()
        });

        widget::container::Container::new(
            Column::with_children(children).align_items(Alignment::Center),
        )
        .style(theme::Container::ContextDrawer)
        .padding([spacing.space_none, spacing.space_xxs])
        .into()
    }
}

async fn load_all_pokemon() -> Vec<NamedApiResource<Pokemon>> {
    let client = rustemon::client::RustemonClient::default();
    rustemon::pokemon::pokemon::get_all_entries(&client)
        .await
        .unwrap_or_default()
}

async fn load_pokemon(pokemon_name: String) -> Pokemon {
    let client = rustemon::client::RustemonClient::default();
    rustemon::pokemon::pokemon::get_by_name(pokemon_name.as_str(), &client)
        .await
        .unwrap_or_default()
}

fn capitalize_string(input: &str) -> String {
    let words: Vec<&str> = input.split('-').collect();

    let capitalized_words: Vec<String> = words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            if let Some(first_char) = chars.next() {
                first_char.to_uppercase().collect::<String>() + chars.as_str()
            } else {
                String::new()
            }
        })
        .collect();

    capitalized_words.join(" ")
}

fn scale_numbers(num: i64) -> f64 {
    (num as f64) / 10.0
}
