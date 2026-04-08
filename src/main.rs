use leptos::ev::PointerEvent;
use leptos::*;
use leptos_router::*;
use rapier2d::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{Clamped, JsCast};

// ==========================================
// SECTION 1 : MANDELBROT FRACTAL
// ==========================================

// Pure Rust function to calculate the fractal image
fn generate_mandelbrot_pixels(
    width: u32,
    height: u32,
    center_x: f64,
    center_y: f64,
    zoom: f64,
    color_offset: u32,
) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    let max_iter = 150;

    let aspect_ratio = width as f64 / height as f64;
    let range_x = 4.0 / zoom * aspect_ratio;
    let range_y = 4.0 / zoom;
    let x_min = center_x - range_x / 2.0;
    let y_min = center_y - range_y / 2.0;

    for y in 0..height {
        for x in 0..width {
            let c_real = x_min + (x as f64 / width as f64) * range_x;
            let c_imag = y_min + (y as f64 / height as f64) * range_y;

            let mut real = c_real;
            let mut imag = c_imag;
            let mut iter = 0;

            // Main Mandelbrot Loop
            // (Note for C++ devs: Rust optimizes this loop in a blazing way.)
            while real * real + imag * imag <= 4.0 && iter < max_iter {
                let next_real = real * real - imag * imag + c_real;
                let next_imag = 2.0 * real * imag + c_imag;
                real = next_real;
                imag = next_imag;
                iter += 1;
            }

            // --- Smooth coloring algorithm ---
            // We calculate a continuous color "index", not just the entire iteration
            if iter == max_iter {
                // Interior of the set: Black
                pixels.push(0);
                pixels.push(0);
                pixels.push(0);
                pixels.push(255);
            } else {
                // Smooth coloring algorithm to get rid of the stripe effect
                // sqrt(real*real + imag*imag) distance to origin
                let log_z = (real * real + imag * imag).ln() / 2.0;
                let nu = (log_z / 2.0_f64.ln()).ln() / 2.0_f64.ln();

                // Smooth coloring index
                let t = (iter as f64 + 1.0 - nu) / max_iter as f64;

                // Generating a dynamic color palette (HSL type)
                // based on trigonometric functions.
                let shift = (color_offset as f64) / 100.0;

                // Different frequencies are used for R, G, B to create gradients.
                let r = (0.5 * (1.0 + (6.28318 * (1.0 * t + shift + 0.00)).cos()) * 255.0) as u8;
                let g = (0.5 * (1.0 + (6.28318 * (1.0 * t + shift + 0.33)).cos()) * 255.0) as u8;
                let b = (0.5 * (1.0 + (6.28318 * (1.0 * t + shift + 0.67)).cos()) * 255.0) as u8;

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(255); // Alpha
            }
        }
    }
    pixels
}

#[component]
fn MandelbrotPage() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let width = 800;
    let height = 600;

    // Reactive signals
    let (center_x, set_center_x) = create_signal(0.0);
    let (center_y, set_center_y) = create_signal(0.625);
    let (zoom, set_zoom) = create_signal(1.0);
    let (color_offset, set_color_offset) = create_signal(5);

    let (is_zooming_in, set_is_zooming_in) = create_signal(true);

    // Effect for drawing the fractal (unchanged)
    create_effect(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            let pixels = generate_mandelbrot_pixels(
                width,
                height,
                center_x.get(),
                center_y.get(),
                zoom.get(),
                color_offset.get(),
            );

            let clamped = Clamped(pixels.as_slice());
            let image_data =
                web_sys::ImageData::new_with_u8_clamped_array_and_sh(clamped, width, height)
                    .unwrap();
            ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
        }
    });

    // --- Canvas click handling ---
    let on_canvas_pointerdown = move |ev: PointerEvent| {
        let canvas = canvas_ref.get().unwrap();

        let scale_x = width as f64 / canvas.client_width() as f64;
        let scale_y = height as f64 / canvas.client_height() as f64;

        let px = ev.offset_x() as f64 * scale_x;
        let py = ev.offset_y() as f64 * scale_y;

        let w = width as f64;
        let h = height as f64;
        let current_zoom = zoom.get();

        let aspect_ratio = w / h;
        let x_range = 4.0 / current_zoom * aspect_ratio;
        let y_range = 4.0 / current_zoom;

        let min_x = center_x.get() - x_range / 2.0;
        let min_y = center_y.get() - y_range / 2.0;

        let clicked_cx = min_x + (px / w) * x_range;
        let clicked_cy = min_y + (py / h) * y_range;

        set_center_x.set(clicked_cx);
        set_center_y.set(clicked_cy);

        // NOUVEAU : On vérifie si la touche MAJ est enfoncée (PC) OU si le bouton mobile est sur "Dézoomer"
        if ev.shift_key() || !is_zooming_in.get() {
            set_zoom.update(|z| *z /= 1.5);
        } else {
            set_zoom.update(|z| *z *= 1.5);
        }
    };

    view! {
        <div class="card">
            <h2>"Explorateur de Mandelbrot"</h2>
            <p>
                "Calculé en temps réel en WebAssembly. "<br/>
                "Clique (ou tapote) sur l'image pour explorer."
            </p>

            <div class="btn-group">
                // Le bouton pour basculer le mode (très pratique sur mobile !)
                <button
                    class="btn"
                    on:click=move |_| set_is_zooming_in.update(|z| *z = !*z)
                    // On change la couleur dynamiquement pour bien montrer le mode actif
                    style=move || if is_zooming_in.get() { "background: var(--accent);" } else { "background: #ef4444;" }
                >
                    {move || if is_zooming_in.get() { "🔍 Mode : Zoom IN" } else { "🔎 Mode : Zoom OUT" }}
                </button>

                <button class="btn btn-secondary" on:click=move |_| { set_zoom.set(1.0); set_center_x.set(-0.5); set_center_y.set(0.0); }>
                    "Reset Vue"
                </button>
                <button class="btn btn-secondary" on:click=move |_| set_color_offset.update(|c| *c += 7)>
                    "Couleurs"
                </button>
            </div>

            <canvas
                node_ref=canvas_ref
                width=width
                height=height
                // NOUVEAU : on utilise pointerdown au lieu de click
                on:pointerdown=on_canvas_pointerdown
                style="cursor: crosshair;"
            ></canvas>
        </div>
    }
}

// ==========================================
// SECTION 2 : PHYSICS ENGINE RAPIER2D
// ==========================================

struct PhysicsState {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
}

impl PhysicsState {
    fn new() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let ground_collider = ColliderBuilder::cuboid(400.0, 20.0).build();
        let ground_rb = RigidBodyBuilder::fixed()
            .translation(vector![300.0, 300.0])
            .build();
        let ground_handle = rigid_body_set.insert(ground_rb);
        collider_set.insert_with_parent(ground_collider, ground_handle, &mut rigid_body_set);

        Self {
            rigid_body_set,
            collider_set,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            gravity: vector![0.0, 1500.0],
            integration_parameters: IntegrationParameters::default(),
        }
    }

    fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    fn add_ball(&mut self, x: f32, y: f32) {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y])
            .build();
        let handle = self.rigid_body_set.insert(rigid_body);
        let collider = ColliderBuilder::ball(7.0).restitution(0.8).build();
        self.collider_set
            .insert_with_parent(collider, handle, &mut self.rigid_body_set);
    }
}

fn draw_physics(ctx: &web_sys::CanvasRenderingContext2d, state: &PhysicsState) {
    ctx.clear_rect(0.0, 0.0, 400.0, 300.0);
    for (_, rb) in state.rigid_body_set.iter() {
        let pos = rb.translation();
        if rb.is_fixed() {
            ctx.set_fill_style_str("#4ADE80");
            ctx.fill_rect(pos.x as f64 - 300.0, pos.y as f64 - 20.0, 400.0, 10.0);
        } else {
            ctx.set_fill_style_str("#3B82F6");
            ctx.begin_path();
            ctx.arc(
                pos.x as f64,
                pos.y as f64,
                8.0,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
            ctx.fill();
        }
    }
}

#[component]
fn PhysicsPage() -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let physics_state = Rc::new(RefCell::new(PhysicsState::new()));

    let state_for_btn = physics_state.clone();
    let add_ball_handler = move |_| {
        let random_offset = (js_sys::Math::random() * 60.0 - 30.0) as f32;
        state_for_btn
            .borrow_mut()
            .add_ball(200.0 + random_offset, 50.0);
    };

    let state_for_loop = physics_state.clone();
    create_effect(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            let value = state_for_loop.clone();
            set_interval_with_handle(
                move || {
                    let mut state = value.borrow_mut();
                    state.step();
                    draw_physics(&ctx, &state);
                },
                std::time::Duration::from_millis(16),
            )
            .expect("Animation loop error");
        }
    });

    view! {
        <div class="card">
            <h2>"Physics Engine Rapier2D"</h2>
            <p>"Dynamic rigid bodies simulation. Collision calculations are performed in Rust."</p>

            <div class="btn-group">
                <button class="btn" on:click=add_ball_handler>"Drop a ball"</button>
            </div>

            <canvas node_ref=canvas_ref width="400" height="300" style="border: 2px solid #333; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);"></canvas>
        </div>
    }
}

// ==========================================
// SECTION 3 : APPLICATION PRINCIPALE ET ROUTEUR
// ==========================================

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <nav>

                // Colonne 1 : Ton texte à droite
                <div class="nav-left">
                    "Welcome to my RUST/WASM demo page!"
                </div>

                // Colonne 2 : Tes liens au centre
                <div class="nav-links">
                    <A href="/" exact=true>"Mandelbrot"</A>
                    <A href="/physics">"Physics Engine"</A>
                </div>

                // Colonne 3 : Le crabe Ferris avec un lien vers l'accueil !
                <div class="nav-logo">
                    <img
                        src="https://rustacean.net/assets/rustacean-flat-happy.svg"
                        alt="Ferris the Rust Crab"
                    />
                </div>
            </nav>

            <main>
                <Routes>
                    <Route path="/" view=MandelbrotPage/>
                    <Route path="/physics" view=PhysicsPage/>
                </Routes>
            </main>
        </Router>
    }
}

fn main() {
    mount_to_body(App)
}
