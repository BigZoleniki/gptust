use macroquad::prelude::*;

struct Player {
    pos: Vec2,
    speed: f32,
    texture: Texture2D,
    health: i32,
    dead: bool,
}

struct Bullet {
    pos: Vec2,
    velocity: Vec2,
    is_enemy: bool,
}

struct Enemy {
    pos: Vec2,
    speed: f32,
    texture: Texture2D,
    health: i32,
    last_shot_time: f32,
}

#[macroquad::main("Top-Down Shooter")]
async fn main() {
    let player_texture = load_texture("assets/player.png").await.unwrap();
    let enemy_texture = load_texture("assets/enemy.png").await.unwrap();

    let mut score = 0;

    // Game state reset function
    let mut reset_game = || {
        let player = Player {
            pos: vec2(screen_width() / 2.0, screen_height() / 2.0),
            speed: 200.0,
            texture: player_texture,
            health: 5,
            dead: false,
        };

        let bullets: Vec<Bullet> = Vec::new();
        let enemies: Vec<Enemy> = vec![
            Enemy {
                pos: vec2(100.0, 100.0),
                speed: 50.0,
                texture: enemy_texture.clone(),
                health: 3,
                last_shot_time: 0.0,
            },
            Enemy {
                pos: vec2(700.0, 400.0),
                speed: 60.0,
                texture: enemy_texture.clone(),
                health: 3,
                last_shot_time: 0.0,
            },
        ];

        (player, bullets, enemies)
    };

    let (mut player, mut bullets, mut enemies) = reset_game();
    let mut player_last_shot_time = 0.0;
    let player_size = 32.0;
    let bullet_size = 6.0;

    loop {
        clear_background(DARKGRAY);
        let dt = get_frame_time();

        // Restart on 'R' if dead
        if player.dead && is_key_pressed(KeyCode::R) {
            let (p, b, e) = reset_game();
            player = p;
            bullets = b;
            enemies = e;
            player_last_shot_time = 0.0;
            score = 0;
        }

        // Mouse aiming
        let mouse_pos = vec2(mouse_position().0, mouse_position().1);
        let aim_dir = {
            let d = mouse_pos - player.pos;
            if d.length_squared() > 0.0 { d.normalize() } else { vec2(1.0, 0.0) }
        };
        let angle = aim_dir.y.atan2(aim_dir.x);

        // Player input if alive
        if !player.dead {
            let mut dir = vec2(0.0, 0.0);
            if is_key_down(KeyCode::W) {
                dir.y -= 1.0;
            }
            if is_key_down(KeyCode::S) {
                dir.y += 1.0;
            }
            if is_key_down(KeyCode::A) {
                dir.x -= 1.0;
            }
            if is_key_down(KeyCode::D) {
                dir.x += 1.0;
            }

            if dir.length() > 0.0 {
                player.pos += dir.normalize() * player.speed * dt;
            }

            if is_mouse_button_down(MouseButton::Left) && player_last_shot_time <= 0.0 {
                bullets.push(Bullet {
                    pos: player.pos + vec2(player_size / 2.0, player_size / 2.0),
                    velocity: aim_dir * 500.0,
                    is_enemy: false,
                });
                player_last_shot_time = 0.2;
            }
        }
        player_last_shot_time -= dt;

        // Enemies
        for enemy in &mut enemies {
            let to_player = player.pos - enemy.pos;
            if to_player.length() > 0.0 {
                enemy.pos += to_player.normalize() * enemy.speed * dt;
            }

            if enemy.last_shot_time <= 0.0 {
                let aim = (player.pos - enemy.pos).normalize();
                bullets.push(Bullet {
                    pos: enemy.pos + vec2(player_size / 2.0, player_size / 2.0),
                    velocity: aim * 300.0,
                    is_enemy: true,
                });
                enemy.last_shot_time = 1.0;
            }
            enemy.last_shot_time -= dt;

            draw_texture_ex(
                &enemy.texture,
                enemy.pos.x,
                enemy.pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(player_size, player_size)),
                    ..Default::default()
                },
            );

            let hp_percent = enemy.health as f32 / 3.0;
            draw_rectangle(enemy.pos.x, enemy.pos.y - 8.0, player_size, 5.0, GRAY);
            draw_rectangle(enemy.pos.x, enemy.pos.y - 8.0, player_size * hp_percent, 5.0, RED);
        }

        // Count enemy deaths
        let prev_enemy_count = enemies.len();
        enemies.retain(|e| e.health > 0);
        score += prev_enemy_count - enemies.len();

        // Spawn new enemies
        if enemies.len() <= 3 {
            enemies.push(Enemy {
                pos: vec2(
                    rand::gen_range(50.0, screen_width() - 50.0),
                    rand::gen_range(50.0, screen_height() - 50.0),
                ),
                speed: rand::gen_range(40.0, 80.0),
                texture: enemy_texture.clone(),
                health: 3,
                last_shot_time: 0.0,
            });
        }

        // Bullets
        for bullet in &mut bullets {
            bullet.pos += bullet.velocity * dt;
            draw_rectangle(
                bullet.pos.x,
                bullet.pos.y,
                bullet_size,
                bullet_size,
                if bullet.is_enemy { RED } else { GREEN },
            );
        }

        bullets.retain(|b| {
            if b.pos.x < 0.0 || b.pos.x > screen_width() || b.pos.y < 0.0 || b.pos.y > screen_height() {
                return false;
            }

            if !b.is_enemy {
                for enemy in &mut enemies {
                    if (enemy.pos - b.pos).length() < player_size / 2.0 {
                        enemy.health -= 1;
                        return false;
                    }
                }
            } else {
                if !player.dead && (player.pos - b.pos).length() < player_size / 2.0 {
                    player.health -= 1;
                    if player.health <= 0 {
                        player.dead = true;
                    }
                    return false;
                }
            }
            true
        });

        // Draw player
        draw_texture_ex(
            &player.texture,
            player.pos.x,
            player.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: angle,
                dest_size: Some(vec2(player_size, player_size)),
                ..Default::default()
            },
        );

        // Player HP bar
        let hp = player.health as f32 / 5.0;
        draw_rectangle(player.pos.x, player.pos.y - 8.0, player_size, 5.0, GRAY);
        draw_rectangle(player.pos.x, player.pos.y - 8.0, player_size * hp, 5.0, GREEN);

        // Score display
        draw_text(&format!("Score: {}", score), 10.0, 20.0, 30.0, WHITE);

        // Game over UI
        if player.dead {
            draw_text("YOU DIED", screen_width() / 2.0 - 80.0, screen_height() / 2.0, 40.0, RED);
            draw_text(
                "Press R to Restart",
                screen_width() / 2.0 - 120.0,
                screen_height() / 2.0 + 40.0,
                30.0,
                WHITE,
            );
        }

        next_frame().await;
    }
}
