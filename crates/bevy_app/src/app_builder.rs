use crate::{
    app::{App, AppExit},
    event::Events,
    plugin::Plugin,
    CoreStage, PluginGroup, PluginGroupBuilder, StartupStage,
};
use bevy_ecs::{
    clear_trackers_system, FromResources, IntoExclusiveSystem, IntoSystem, Resource, Resources,
    RunOnce, Schedule, Stage, StageLabel, StateStage, SystemDescriptor, SystemSet, SystemStage,
    World,
};
use bevy_utils::tracing::debug;

/// Configure [App]s using the builder pattern
pub struct AppBuilder {
    pub app: App,
}

impl Default for AppBuilder {
    fn default() -> Self {
        let mut app_builder = AppBuilder {
            app: App::default(),
        };

        app_builder
            .add_default_stages()
            .add_event::<AppExit>()
            .add_system_to_stage(CoreStage::Last, clear_trackers_system.exclusive_system());
        app_builder
    }
}

impl AppBuilder {
    pub fn empty() -> AppBuilder {
        AppBuilder {
            app: App::default(),
        }
    }

    pub fn resources(&self) -> &Resources {
        &self.app.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.app.resources
    }

    /// Start the application (through main runner)
    ///
    /// Runs the application main loop.
    ///
    /// Usually the main loop is handled by bevy integrated plugins (`winit`), but
    /// but one can also set the runner function through [`AppBuilder::set_runner`].
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// App::build()
    ///     // all required plugin insertions, systems, etc inserted here
    ///     // finally, call:
    ///     .run();
    /// ```
    pub fn run(&mut self) {
        let app = std::mem::take(&mut self.app);
        app.run();
    }

    pub fn set_world(&mut self, world: World) -> &mut Self {
        self.app.world = world;
        self
    }

    pub fn add_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
        self.app.schedule.add_stage(label, stage);
        self
    }

    pub fn add_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.app.schedule.add_stage_after(target, label, stage);
        self
    }

    pub fn add_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.app.schedule.add_stage_before(target, label, stage);
        self
    }

    pub fn add_startup_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
        self.app
            .schedule
            .stage(CoreStage::Startup, |schedule: &mut Schedule| {
                schedule.add_stage(label, stage)
            });
        self
    }

    pub fn add_startup_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.app
            .schedule
            .stage(CoreStage::Startup, |schedule: &mut Schedule| {
                schedule.add_stage_after(target, label, stage)
            });
        self
    }

    pub fn add_startup_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        self.app
            .schedule
            .stage(CoreStage::Startup, |schedule: &mut Schedule| {
                schedule.add_stage_before(target, label, stage)
            });
        self
    }

    pub fn stage<T: Stage, F: FnOnce(&mut T) -> &mut T>(
        &mut self,
        label: impl StageLabel,
        func: F,
    ) -> &mut Self {
        self.app.schedule.stage(label, func);
        self
    }

    /// Adds a system that runs every time `app.update()` is called by the runner
    ///
    /// Systems are the main building block in bevy ECS model. You can define
    /// normal rust functions, and call `.system()` to make them be bevy systems.
    ///
    /// System functions can have parameters, through which one can query and
    /// mutate bevy ECS states. See bevy book for extra information.
    ///
    /// Systems are run in parallel, and the execution order is not deterministic.
    /// If you want more fine-grained control for order, see [`AppBuilder::add_system_to_stage`].
    ///
    /// For adding a system that runs only at app startup, see [`AppBuilder::add_startup_system`].
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// # use bevy_ecs::prelude::*;
    /// #
    /// fn my_system(_commands: &mut Commands) {
    ///     println!("My system, triggered once per frame");
    /// }
    ///
    /// App::build()
    ///     .add_system(my_system.system());
    /// ```
    pub fn add_system(&mut self, system: impl Into<SystemDescriptor>) -> &mut Self {
        self.add_system_to_stage(CoreStage::Update, system)
    }

    pub fn add_system_set(&mut self, system_set: SystemSet) -> &mut Self {
        self.add_system_set_to_stage(CoreStage::Update, system_set)
    }

    pub fn add_system_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system: impl Into<SystemDescriptor>,
    ) -> &mut Self {
        self.app.schedule.add_system_to_stage(stage_label, system);
        self
    }

    pub fn add_system_set_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system_set: SystemSet,
    ) -> &mut Self {
        self.app
            .schedule
            .add_system_set_to_stage(stage_label, system_set);
        self
    }

    /// Adds a system that is run once at application startup
    ///
    /// Startup systems run exactly once BEFORE all other systems. These are generally used for
    /// app initialization code (ex: adding entities and resources).
    ///
    /// * For adding a system that runs for every frame, see [`AppBuilder::add_system`].
    /// * For adding a system to specific stage, see [`AppBuilder::add_system_to_stage`].
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// # use bevy_ecs::prelude::*;
    /// #
    /// fn my_startup_system(_commands: &mut Commands) {
    ///     println!("My startup system");
    /// }
    ///
    /// App::build()
    ///     .add_startup_system(my_startup_system.system());
    /// ```
    pub fn add_startup_system(&mut self, system: impl Into<SystemDescriptor>) -> &mut Self {
        self.add_startup_system_to_stage(StartupStage::Startup, system)
    }

    pub fn add_startup_system_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system: impl Into<SystemDescriptor>,
    ) -> &mut Self {
        self.app
            .schedule
            .stage(CoreStage::Startup, |schedule: &mut Schedule| {
                schedule.add_system_to_stage(stage_label, system)
            });
        self
    }

    pub fn on_state_enter<T: Clone + Resource>(
        &mut self,
        stage: impl StageLabel,
        state: T,
        system: impl Into<SystemDescriptor>,
    ) -> &mut Self {
        self.stage(stage, |stage: &mut StateStage<T>| {
            stage.on_state_enter(state, system)
        })
    }

    pub fn on_state_update<T: Clone + Resource>(
        &mut self,
        stage: impl StageLabel,
        state: T,
        system: impl Into<SystemDescriptor>,
    ) -> &mut Self {
        self.stage(stage, |stage: &mut StateStage<T>| {
            stage.on_state_update(state, system)
        })
    }

    pub fn on_state_exit<T: Clone + Resource>(
        &mut self,
        stage: impl StageLabel,
        state: T,
        system: impl Into<SystemDescriptor>,
    ) -> &mut Self {
        self.stage(stage, |stage: &mut StateStage<T>| {
            stage.on_state_exit(state, system)
        })
    }

    pub fn add_default_stages(&mut self) -> &mut Self {
        self.add_stage(
            CoreStage::Startup,
            Schedule::default()
                .with_run_criteria(RunOnce::default())
                .with_stage(StartupStage::PreStartup, SystemStage::parallel())
                .with_stage(StartupStage::Startup, SystemStage::parallel())
                .with_stage(StartupStage::PostStartup, SystemStage::parallel()),
        )
        .add_stage(CoreStage::First, SystemStage::parallel())
        .add_stage(CoreStage::PreEvent, SystemStage::parallel())
        .add_stage(CoreStage::Event, SystemStage::parallel())
        .add_stage(CoreStage::PreUpdate, SystemStage::parallel())
        .add_stage(CoreStage::Update, SystemStage::parallel())
        .add_stage(CoreStage::PostUpdate, SystemStage::parallel())
        .add_stage(CoreStage::Last, SystemStage::parallel())
    }

    pub fn add_event<T>(&mut self) -> &mut Self
    where
        T: Send + Sync + 'static,
    {
        self.insert_resource(Events::<T>::default())
            .add_system_to_stage(CoreStage::Event, Events::<T>::update_system.system())
    }

    /// Inserts a resource to the current [App] and overwrites any resource previously added of the same type.
    ///
    /// A resource in bevy represents globally unique data. The resources must be added to bevy application
    /// before using them. This happens with [`AppBuilder::insert_resource`].
    ///
    /// For adding a main-thread only accessible resource, see [`AppBuilder::insert_non_send_resource`].
    ///
    /// See also `init_resource` for resources that implement `Default` or [`FromResources`].
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// struct State {
    ///     counter: usize,
    /// }
    ///
    /// App::build()
    ///    .insert_resource(State { counter: 0 });
    /// ```
    pub fn insert_resource<T>(&mut self, resource: T) -> &mut Self
    where
        T: Send + Sync + 'static,
    {
        self.app.resources.insert(resource);
        self
    }

    /// Inserts a non-send resource to the app
    ///
    /// You usually want to use `insert_resource`, but there are some special cases when a resource must
    /// be non-send.
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// struct State {
    ///     counter: usize,
    /// }
    ///
    /// App::build()
    ///     .insert_non_send_resource(State { counter: 0 });
    /// ```
    pub fn insert_non_send_resource<T>(&mut self, resource: T) -> &mut Self
    where
        T: 'static,
    {
        self.app.resources.insert_non_send(resource);
        self
    }

    /// Init a resource to the current [App], if it does not exist yet
    ///
    /// Adds a resource that implements `Default` or [`FromResources`] trait.
    /// If the resource already exists, `init_resource` does nothing.
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// struct State {
    ///     counter: usize,
    /// }
    ///
    /// impl Default for State {
    ///     fn default() -> State {
    ///         State {
    ///             counter: 100
    ///         }
    ///     }
    /// }
    ///
    /// App::build()
    ///     .init_resource::<State>();
    /// ```
    pub fn init_resource<R>(&mut self) -> &mut Self
    where
        R: FromResources + Send + Sync + 'static,
    {
        // PERF: We could avoid double hashing here, since the `from_resources` call is guaranteed not to
        // modify the map. However, we would need to be borrowing resources both mutably and immutably,
        // so we would need to be extremely certain this is correct
        if !self.resources().contains::<R>() {
            let resource = R::from_resources(&self.resources());
            self.insert_resource(resource);
        }
        self
    }

    pub fn init_non_send_resource<R>(&mut self) -> &mut Self
    where
        R: FromResources + 'static,
    {
        // See perf comment in init_resource
        if self.app.resources.get_non_send::<R>().is_none() {
            let resource = R::from_resources(&self.app.resources);
            self.app.resources.insert_non_send(resource);
        }
        self
    }

    /// Sets the main runner loop function for bevy application
    ///
    /// Usually the main loop is handled by bevy integrated plugins (`winit`), but
    /// in some cases one wants to implement an own main loop.
    ///
    /// This method sets the main loop function, overwriting a previous runner if any.
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// fn my_runner(mut app: App) {
    ///     loop {
    ///         println!("In main loop");
    ///         app.update();
    ///     }
    /// }
    ///
    /// App::build()
    ///     .set_runner(my_runner);
    /// ```
    pub fn set_runner(&mut self, run_fn: impl Fn(App) + 'static) -> &mut Self {
        self.app.runner = Box::new(run_fn);
        self
    }

    /// Adds a single plugin
    ///
    /// One of Bevy's core principles is modularity. All bevy engine features are implemented
    /// as plugins. This includes internal features like the renderer.
    ///
    /// Bevy also provides a few sets of default plugins. See [`AppBuilder::add_plugins`].
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// App::build().add_plugin(bevy_log::LogPlugin::default());
    /// ```
    pub fn add_plugin<T>(&mut self, plugin: T) -> &mut Self
    where
        T: Plugin,
    {
        debug!("added plugin: {}", plugin.name());
        plugin.build(self);
        self
    }

    /// Adds a group of plugins
    ///
    /// Bevy plugins can be grouped into a set of plugins. By default
    /// bevy provides a few lists of plugins that can be used to kickstart
    /// the development.
    ///
    /// Current plugins offered are [`DefaultPlugins`] and [`MinimalPlugins`].
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// App::build();
    ///     //.add_plugins(MinimalPlugins)
    /// ```
    pub fn add_plugins<T: PluginGroup>(&mut self, mut group: T) -> &mut Self {
        let mut plugin_group_builder = PluginGroupBuilder::default();
        group.build(&mut plugin_group_builder);
        plugin_group_builder.finish(self);
        self
    }

    /// Adds a group of plugins with an initializer method
    ///
    /// Can be used to add a group of plugins, where the group is modified
    /// before insertion into Bevy application. For example, you can add
    /// extra plugins at a specific place in the plugin group, or deactivate
    /// specific plugins while keeping the rest.
    ///
    /// ## Example
    /// ```
    /// # use bevy_app::prelude::*;
    /// #
    /// App::build();
    ///     // .add_plugins_with(DefaultPlugins, |group| {
    ///            // group.add_before::<bevy::asset::AssetPlugin, _>(MyOwnPlugin)
    ///        // })
    /// ```
    pub fn add_plugins_with<T, F>(&mut self, mut group: T, func: F) -> &mut Self
    where
        T: PluginGroup,
        F: FnOnce(&mut PluginGroupBuilder) -> &mut PluginGroupBuilder,
    {
        let mut plugin_group_builder = PluginGroupBuilder::default();
        group.build(&mut plugin_group_builder);
        func(&mut plugin_group_builder);
        plugin_group_builder.finish(self);
        self
    }
}
