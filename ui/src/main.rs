use leptos::*;
use leptos_router::*;
use reqwasm::http::Request;
use thaw::*;

#[component]
fn Home() -> impl IntoView {
    view! {
        <h1>"What's this?"</h1>
        <p>Create quick and easy redirects</p>
    }
}

mod input_validation {
    pub fn name(val: String) -> bool {
        val != "_internal" && !val.contains('/')
    }

    pub fn url(val: String) -> bool {
        val.starts_with("http")
    }
}

#[inline]
fn validated_input(value: Signal<String>, validation_cb: impl Fn(String) -> bool + 'static) -> Signal<bool> {
    Signal::derive(move || !validation_cb(value.get()))
}

#[component]
fn New() -> impl IntoView {
    let message = use_message();
    let name = RwSignal::new(String::new());
    let url = RwSignal::new(String::new());
    let create_link = create_action(move |(name, url): &(String, String)| {
        let name = name.clone();
        let url = url.clone();
        async move {
            match Request::get(&format!("/_internal/new?name={name}&to={url}"))
                .send()
                .await
            {
                Ok(resp) => match resp.status() {
                    201 => message.create(
                        "Success in creating redirect!".into(),
                        MessageVariant::Success,
                        Default::default(),
                    ),
                    409 => message.create(
                        format!("{name} already exists"),
                        MessageVariant::Error,
                        Default::default(),
                    ),
                    502 => {
                        let msg = resp
                            .text()
                            .await
                            .unwrap_or("<failed to obtain reason>".into());
                        message.create(
                            format!("Server said no! Because: {msg}",),
                            MessageVariant::Error,
                            MessageOptions {
                                duration: std::time::Duration::from_secs(0),
                                closable: true,
                            },
                        )
                    }
                    x => message.create(
                        format!("Unknown error code: {x}"),
                        MessageVariant::Error,
                        Default::default(),
                    ),
                },
                Err(err) => message.create(
                    format!("Could not talk to server: {err}"),
                    MessageVariant::Error,
                    Default::default(),
                ),
            }
        }
    });
    view! {
        <h1>"Something new"</h1>
            <Space>
                <Space vertical=true>
                    <Space>
                        <Input value=name invalid={validated_input(name.into(), input_validation::name)} placeholder="name">
                            <InputPrefix slot>
                                <Icon icon=icondata::AiTagOutlined />
                            </InputPrefix>
                        </Input>
                        <Input value=url invalid={validated_input(url.into(), input_validation::url)} placeholder="url" attr:size=50>
                            <InputPrefix slot>
                                <Icon icon=icondata::AiLinkOutlined />
                            </InputPrefix>
                        </Input>
                    </Space>
                    <Button variant=ButtonVariant::Primary on_click=move |_| create_link.dispatch((name.get(), url.get())) block=true>Create!</Button>
                </Space>
            </Space>
    }
}

#[component]
fn Content() -> impl IntoView {
    let navigate = use_navigate();
    let selected = RwSignal::new("home".to_string());
    create_effect(move |_| {
        let sel = use_location()
            .pathname
            .get()
            .strip_prefix("/_internal/ui/")
            .unwrap()
            .to_owned();
        if sel.is_empty() {
            selected.set("home".to_string());
        } else {
            selected.set(sel);
        }
    });
    _ = selected.watch(move |name| {
        navigate(&format!("/_internal/ui/{name}"), Default::default());
    });
    let menu = RwSignal::new(true);
    let toggle_menu = Callback::from(move |_| { let old = menu.get_untracked(); menu.set(!old); });
    view! {
        <Layout position=LayoutPosition::Absolute>
            <LayoutHeader style="border-bottom: 1px solid grey; padding-left: 0.5rem;">
                <Space align=SpaceAlign::Center gap=SpaceGap::Large>
                    <Icon width="2em" height="2em" icon=icondata::AiMenuOutlined on_click=toggle_menu style="cursor: pointer;" />
                    <h1>Oxidized Redirects</h1>
                </Space>
            </LayoutHeader>
            <Layout has_sider=menu>
                <Show when=move || menu.get() fallback=|| view! {}>
                    <LayoutSider style="border-right: 1px solid #eee;">
                        <Menu value=selected>
                            <MenuItem icon=icondata::AiHomeOutlined key="home" label="Home" />
                            <MenuItem icon=icondata::AiPlusOutlined key="new" label="New" />
                        </Menu>
                    </LayoutSider>
                </Show>
                <Layout style="margin-left: 1rem;">
                    <MessageProvider>
                        <Routes>
                            <Route path="/_internal/ui/*any" view=Home />
                            <Route path="/_internal/ui/new" view=New />
                        </Routes>
                    </MessageProvider>
                </Layout>
            </Layout>
        </Layout>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <ThemeProvider theme=Theme::light()>
                <GlobalStyle />
                <Content />
            </ThemeProvider>
        </Router>
    }
}

fn main() {
    mount_to_body(App)
}
