use macroquad::prelude::{*};
use std::collections::HashMap;
fn window_config() -> Conf {
    Conf {
        window_title: String::from("RTS"),
        fullscreen: false,
        window_resizable: false,
        window_height: 420,
        window_width: 420,
        sample_count: 2,
        ..Default::default()
    }
}
struct Resources {
    textures_tank: Vec<Texture2D>,
    texture_map: Texture2D,
}
#[derive(Clone, Copy)]
struct Rect {
    center_position: Vec2,
    half_extents: Vec2,
}
impl Rect {
    fn new(center_position: Vec2, half_extents: Vec2) -> Rect {
        Rect {
            center_position,
            half_extents,
        }
    }
    fn from_points(point1: Vec2, point2: Vec2) -> Rect {
        let min = Vec2::min(point1, point2);
        let max = Vec2::max(point1, point2);
        let half_extents = (max - min) / 2.0;
        let center = min + half_extents;
        Rect {
            center_position: center,
            half_extents,
        }
    }
    fn extents(&self) -> Vec2 {
        self.half_extents * 2.0
    }
    fn min_point(&self) -> Vec2 {
        self.center_position - self.half_extents
    }
    fn max_point(&self) -> Vec2 {
        self.center_position + self.half_extents
    }
    fn do_rectangles_collide(&self, rect2: Rect) -> bool {
        let abs_distance = Vec2::abs(self.center_position - rect2.center_position);
        let half_extents = self.half_extents + rect2.half_extents;
        abs_distance.x <= half_extents.x && abs_distance.y <= half_extents.y
    }
    fn draw(&self) {
        draw_rectangle(
            self.min_point().x,
            self.min_point().y,
            self.extents().y,
            self.extents().y,
            WHITE,
        )
    }
}

fn get_tank_sprite(velocity: Vec2) -> usize {
    // 4 is upward, 2, downward, 1 right, 3 left direction
    if velocity.x.abs() > velocity.y.abs() {
        if velocity.x > 0.0 {
            return 1;
        } else {
            return 3;
        };
    } else if velocity.y.abs() > velocity.x.abs() {
        if velocity.y > 0.0 {
            return 2;
        } else {
            return 4;
        };
    }
    return 4; // default state
}
impl Resources {
    fn new() -> Resources {
        let mut vec_tanks = Vec::new();
        for i in 1..=4 {
            let images = std::fs::read(format!("./images/tank{i}.png")).expect("Failed to load");
            let texture = Texture2D::from_file_with_format(&images, None);
            texture.set_filter(FilterMode::Nearest);
            vec_tanks.push(texture);
        }
        let images = std::fs::read(format!("./images/background.png")).expect("Failed to load");
        let texture_map = Texture2D::from_file_with_format(&images, None);
        texture_map.set_filter(FilterMode::Nearest);
        Resources {
            textures_tank: vec_tanks,
            texture_map: texture_map,
        }
    }
}
struct RectSelect {
    start_point: Vec2,
    end_point: Vec2,
    rect: Rect,
}
enum Object_Type {
    Tank,
   Structure,
}
struct Object {
    object_type: Object_Type,
    flagged_for_collision_check: bool,
}
struct Grid_Cell {
   width: u32,
   height: u32,
   objects: Vec<Object>, 
}
struct Game {
    resources: Resources,
    tanks: Vec<Tank>,
    rect_select: Option<RectSelect>,
    //objects: Vec<Object>,
}
impl Game {
    fn new() -> Game {
        Game {
            resources: Resources::new(),
            tanks: Vec::new(),
            rect_select: None,
        }
    }
    fn update(&mut self, deltatime: f32) {
        let mut mouse_event = Mouse_Event::None;

        if is_key_pressed(KeyCode::S) {
            self.tanks.push(Tank::new(50.0, 150.0));
        }
        let mouse_pos = mouse_position().into();
        if is_mouse_button_pressed(MouseButton::Left) {
            self.rect_select = Some(RectSelect {
                start_point: mouse_pos,
                end_point: mouse_pos,
                rect: Rect::from_points(mouse_position().into(), mouse_position().into()),
            });
        }
        if let Some(rect_select) = &mut self.rect_select {
            rect_select.end_point = mouse_position().into();
            if is_mouse_button_released(MouseButton::Left) {
                // TODO: Select units
                rect_select.rect =
                    Rect::from_points(rect_select.start_point, rect_select.end_point);
                mouse_event = Mouse_Event::Draw_Select;
                for tank in &mut self.tanks {
                    tank.update(
                        deltatime,
                        &self.resources,
                        mouse_event,
                        Some(&rect_select.rect),
                    );
                }
                self.rect_select = None;
            }
        } else if is_mouse_button_pressed(MouseButton::Right) {
            mouse_event = Mouse_Event::Set_Goal;
        } else {
            mouse_event = Mouse_Event::None;
        }
        for tank in &mut self.tanks {
            tank.update(deltatime, &self.resources, mouse_event, None);
        }
    }
    fn draw(&self) {
        draw_texture(self.resources.texture_map, 0.0, 0.0, WHITE);
        if let Some(rect_select) = &self.rect_select {
            draw_rectangle(
                rect_select.start_point.x,
                rect_select.start_point.y,
                rect_select.end_point.x - rect_select.start_point.x,
                rect_select.end_point.y - rect_select.start_point.y,
                WHITE,
            );
        }
        for tank in &self.tanks {
            tank.draw(&self.resources);
        }
    }
}
struct Circle {
    center_position: Vec2,
    radius: Vec2,   // For easier comparison with other rectangles
}
impl Circle {
    fn new(center_position: Vec2, radius: f32) -> Circle {
        Circle {
            center_position,
            radius: Vec2::new(radius, radius),
        }
    }
    fn does_circle_collide_rect(&self, rect: Rect) -> bool {
        let abs_distance = Vec2::abs(self.center_position - rect.center_position);
        let total_extents = self.radius + rect.half_extents;
        abs_distance.x <= total_extents.x && abs_distance.y <= total_extents.y
    }
    fn draw(&self) {
        draw_circle(self.center_position.x, self.center_position.y, self.radius.x, RED);
    }
}
struct Tank {
    position: Vec2,
    velocity: Vec2,
    selected: bool,
    goal_position: Vec2,
    tank_hitbox: Rect,
    tank_attack_range: Circle,
}

fn get_vector_velocity(current_position: Vec2, goal_position: Vec2) -> Vec2 {
    if goal_position.x == 0.0 && goal_position.y == 0.0 {
        return Vec2::new(0.0, 0.0);
    }
    let x = goal_position.x - current_position.x;
    let y = goal_position.y - current_position.y;
    return Vec2::new(x, y);
}
#[derive(Clone, Copy)]
enum Mouse_Event {
    Draw_Select,
    None,
    Set_Goal,
}

impl Tank {
    fn new(x: f32, y: f32) -> Tank {
        Tank {
            position: Vec2::new(x, y),
            velocity: Vec2::ZERO,
            selected: false,
            goal_position: Vec2::ZERO,
            tank_hitbox : Rect::new(Vec2::ZERO, Vec2::ZERO),
            tank_attack_range: Circle::new(Vec2::ZERO, 0.0),
        }
    }
    fn update(
        &mut self,
        deltatime: f32,
        resources: &Resources,
        mouse_event: Mouse_Event,
        rect: Option<&Rect>,
    ) {
        let which_tank_sprite = get_tank_sprite(self.velocity) - 1;
        let velocity = get_vector_velocity(self.position, self.goal_position.into());
        let tank_half_extents = (
            resources.textures_tank[which_tank_sprite].width() * 0.5,   //half extents
            resources.textures_tank[which_tank_sprite].height() * 0.5,  //half extents
        );
        self.tank_hitbox = Rect::new(self.position, tank_half_extents.into());
        match mouse_event {
            Mouse_Event::Draw_Select => {
                if let Some(rect) = rect {
                    if self.tank_hitbox.do_rectangles_collide(*rect) {
                        self.selected = true;
                    }
                }
            }
            Mouse_Event::Set_Goal => {
                if self.selected {
                    self.goal_position = mouse_position().into();
                    self.selected = false;
                }
            }
            Mouse_Event::None => {
                if !self.selected {
                    self.velocity = velocity;
                }
                self.position.x += self.velocity.x * deltatime;
                self.position.y += self.velocity.y * deltatime;

            }
        }
    }
    fn draw(&self, resources: &Resources) {
        let which_tank_sprite = get_tank_sprite(self.velocity) - 1; // adjust for 0 index
        draw_texture(
            resources.textures_tank[which_tank_sprite],
            self.position.x - resources.textures_tank[which_tank_sprite].width() * 0.5, // draw it from center of tank
            self.position.y - resources.textures_tank[which_tank_sprite].height() * 0.5,
            WHITE,
        );
        if self.selected {
            draw_circle(self.position.x, self.position.y, 5.0, RED);
        }
    }
}
fn min(x1: f32, x2: f32) -> f32 {
    if x1 < x2 {
        return x1;
    } else {
        return x2;
    }
}
fn max(x1: f32, x2: f32) -> f32 {
    if x1 > x2 {
        return x1;
    } else {
        return x2;
    }
}
#[macroquad::main(window_config)]
async fn main() {
    let mut game = Game::new();

    loop {
        game.update(get_frame_time());
        game.draw();
        next_frame().await;
    }
}
