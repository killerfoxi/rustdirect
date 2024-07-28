use material_yew::{
    button::MatButton,
    drawer::{MatDrawer, MatDrawerAppContent, MatDrawerTitle},
    list::{ListIndex, MatList, MatListItem},
    text_inputs::MatTextField,
    top_app_bar_fixed::{MatTopAppBarFixed, MatTopAppBarNavigationIcon, MatTopAppBarTitle},
    MatIconButton, WeakComponentLink,
};
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Nav {
    #[at("/")]
    Main,
    #[at("/new")]
    New,
}

#[function_component(Menu)]
fn menu() -> Html {
    let navigator = use_navigator().unwrap();
    let onaction = Callback::from(move |idx| {
        if let ListIndex::Single(Some(idx)) = idx {
            match idx {
                0 => navigator.push(&Nav::Main),
                1 => navigator.push(&Nav::New),
                _ => (),
            }
        }
    });
    html! {
        <MatList onaction={onaction}>
            <MatListItem>{"Home"}</MatListItem>
            <MatListItem>{"New"}</MatListItem>
        </MatList>
    }
}

#[derive(Properties, PartialEq)]
struct ContentProps {
    pub children: Children,
}

#[function_component(Content)]
fn content(prop: &ContentProps) -> Html {
    let drawer_link = use_state(WeakComponentLink::<MatDrawer>::default);
    let nav_click = {
        let drawer_link = drawer_link.clone();
        Callback::from(move |_| drawer_link.flip_open_state())
    };
    html! {
        <MatDrawer drawer_type="dismissible" drawer_link={(*drawer_link).clone()}>
            <MatDrawerTitle>{"Menu"}</MatDrawerTitle>
            <div>
                <Menu />
            </div>
            <MatDrawerAppContent>
                <MatTopAppBarFixed onnavigationiconclick={nav_click}>
                    <MatTopAppBarNavigationIcon><MatIconButton icon="menu" /></MatTopAppBarNavigationIcon>
                    <MatTopAppBarTitle>{"Oxidized Redirect"}</MatTopAppBarTitle>
                </MatTopAppBarFixed>
                {for prop.children.iter()}
            </MatDrawerAppContent>
        </MatDrawer>
    }
}

#[function_component(CreateNew)]
fn create_new() -> Html {
    use reqwasm::http::Request;
    let name = use_state(String::new);
    let url = use_state(String::new);
    let onclick = {
        let name = name.clone();
        let url = url.clone();
        Callback::from(move |_| {
            let name = (*name).clone();
            let url = (*url).clone();
            spawn_local(async move {
                let _ = Request::get(&format!("/_internal/new?name={}&to={}", name, url))
                    .send()
                    .await;
            });
        })
    };
    html! {
        <>
            <div>
                <MatTextField
                    outlined=true
                    required=true
                    label="Name"
                    value={(*name).clone()}
                    oninput={let name = name.clone(); Callback::from(move |e: String| name.set(e))}
                />
                {" "}
                <MatTextField
                    outlined=true
                    required=true
                    label="Url"
                    size=50
                    value={(*url).clone()}
                    oninput={let url = url.clone(); Callback::from(move |e: String| url.set(e))}
                />
            </div>
            <div style="margin-top: 0.5rem;">
                <span onclick={onclick}>
                    <MatButton
                        label="Create"
                        raised=true
                    />
                </span>
            </div>
        </>
    }
}

fn switch(routes: Nav) -> Html {
    match routes {
        Nav::Main => html! {
            <Content>
                <h1>{"What?"}</h1>
                <p>{"Tired of remembering urls? Create a convenience name for it!"}</p>
            </Content>
        },
        Nav::New => html! {
            <Content>
                <h1>{"Creating new redirect"}</h1>
                <CreateNew />
            </Content>
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Nav> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
