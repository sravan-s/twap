use gtk::prelude::{BoxExt, GtkWindowExt};
use relm4::{
    component::{AsyncComponentParts, SimpleAsyncComponent},
    gtk, AsyncComponentSender, RelmApp, RelmWidgetExt,
};
use sysinfo::System;

#[derive(Debug, Clone)]
struct StaticAssets {
    name: String,
    kernel_version: String,
    os_version: String,
    host_name: String,
    cpus: usize,
}

struct AppModel {
    static_assets: StaticAssets,
}

#[derive(Debug)]
enum AppUpdate {
    Init,
}

struct AppWidgets {
    label: gtk::Label,
}

fn static_label(sa: StaticAssets) -> String {
    format!(
        "System Name: {} \n
        Kernel Version: {} \n
        OS Version: {} \n
        Host Name: {} \n
        No.of. CPUs: {}
        ",
        sa.name, sa.kernel_version, sa.os_version, sa.host_name, sa.cpus,
    )
}

impl SimpleAsyncComponent for AppModel {
    /// The type of the messages that this component can receive.
    type Input = AppUpdate;
    /// The type of the messages that this component can send.
    type Output = ();
    /// The type of data with which this component will be initialized.
    type Init = StaticAssets;
    /// The root GTK widget that this component will create.
    type Root = gtk::Window;
    /// A data structure that contains the widgets that you will need to update.
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("twap")
            .default_width(800)
            .default_height(600)
            .build()
    }
    async fn init(
        initial_data: Self::Init,
        window: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = AppModel {
            static_assets: initial_data.clone(),
        };
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .build();

        let label = gtk::Label::new(Some(&static_label(initial_data)));
        label.set_margin_all(1);
        window.set_child(Some(&vbox));
        vbox.set_margin_all(5);

        vbox.append(&label);
        let widgets = AppWidgets { label };

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {}

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: AsyncComponentSender<Self>) {
        widgets
            .label
            .set_label(&static_label(self.static_assets.clone()));
    }
}

fn main() {
    let sys = System::new_all();
    // Display system information:
    let app = RelmApp::new("relm4.test.simple_manual");
    app.run_async::<AppModel>(StaticAssets {
        name: System::name().unwrap(),
        cpus: sys.cpus().len(),
        host_name: System::host_name().unwrap(),
        kernel_version: System::kernel_version().unwrap(),
        os_version: System::os_version().unwrap(),
    });
}
