use macroquad::prelude::*;

struct Player {
    pos: Vec2,
    speed: f32,
    texture: Texture2D,
    health: i32,
}

struct Bullet {
    pos: Vec2,
    velocity: Vec2,
    is_enemy: bool, // Flag to differentiate between player and enemy bullets
}

struct Enemy {
    pos: Vec2,
    speed: f32,
    texture: Texture2D,
    health: i32, // Health for the enemy
    last_shot_time: f32, // Time of last shot
}

#[macroquad::main("Top-Down Shooter")]
async fn main() {
    // Load textures
    let player_texture = load_texture("assets/player.png").await.unwrap();
    let enemy_texture = load_texture("assets/enemy.png").await.unwrap();

    // Create player
    let mut player = Player {
        pos: vec2(screen_width() / 2.0, screen_height() / 2.0),
        speed: 200.0,
        texture: player_texture,
        health: 5, // Starting health for the player
    };

    // Bullets and enemies
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut enemies: Vec<Enemy> = vec![
        Enemy {
            pos: vec2(100.0, 100.0),
            speed: 50.0,
            texture: enemy_texture.clone(),
            health: 3, // Starting health for the enemy
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

    let mut player_last_shot_time = 0.0; // Last time player shot

    let player_size = 32.0; // Same size for both player and enemies
    let bullet_size = 10.0; // Bullet size

    loop {
        clear_background(DARKGRAY);

        let dt = get_frame_time();

        // --- Player movement
        let mut direction = vec2(0.0, 0.0);
        if is_key_down(KeyCode::W) {
            direction.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            direction.y += 1.0;
        }
        if is_key_down(KeyCode::A) {
            direction.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            player.pos += direction.normalize() * player.speed * dt;
        }

        // --- Mouse aiming
        let mouse_pos: Vec2 = vec2(mouse_position().0, mouse_position().1); // Explicit conversion
        let aim_dir: Vec2 = mouse_pos - player.pos; // Explicitly typing as Vec2

        // Check if the vector has a non-zero length before normalizing
        let aim_dir = if aim_dir.length_squared() > 0.0 {
            aim_dir.normalize()
        } else {
            vec2(1.0, 0.0) // Default to a direction if mouse is exactly on player
        };

        let angle = aim_dir.y.atan2(aim_dir.x);

        // --- Fire player bullet with delay
        if is_mouse_button_pressed(MouseButton::Left) && player_last_shot_time <= 0.0 {
            bullets.push(Bullet {
                pos: player.pos + vec2(player_size / 2.0, player_size / 2.0), // Offset from center
                velocity: aim_dir * 500.0,
                is_enemy: false, // Mark this as a player's bullet
            });
            player_last_shot_time = 0.2; // Set firing delay (0.2 seconds)
        }

        // --- Decrease firing delay for player
        player_last_shot_time -= dt;

        // --- Enemies firing bullets with delay
        for enemy in &mut enemies {
            // Simple AI for enemy bullet firing towards the player
            if enemy.last_shot_time <= 0.0 {
                let enemy_aim_dir = player.pos - enemy.pos;
                if enemy_aim_dir.length() > 50.0 { // Avoid firing if too close
                    let aim_dir = enemy_aim_dir.normalize();
                    bullets.push(Bullet {
                        pos: enemy.pos + vec2(player_size / 2.0, player_size / 2.0), // Offset from center
                        velocity: aim_dir * 300.0, // Slower than player's bullet
                        is_enemy: true, // Mark this as an enemy's bullet
                    });
                    enemy.last_shot_time = 1.0; // Set firing delay (1 second)
                }
            }

            // Decrease firing delay for enemies
            enemy.last_shot_time -= dt;
        }

        // --- Update and draw bullets
        for bullet in &mut bullets {
            bullet.pos += bullet.velocity * dt;

            // Draw bullets as rectangles (pixels)
            if bullet.is_enemy {
                draw_rectangle(bullet.pos.x, bullet.pos.y, bullet_size, bullet_size, RED); // Enemy bullets
            } else {
                draw_rectangle(bullet.pos.x, bullet.pos.y, bullet_size, bullet_size, GREEN); // Player bullets
            }
        }

        // --- Bullet collision detection
        bullets.retain(|bullet| {
            // Remove bullets that go out of bounds
            if bullet.pos.x < 0.0 || bullet.pos.x > screen_width() || bullet.pos.y < 0.0 || bullet.pos.y > screen_height() {
                return false;
            }

            // Check collision between player's bullet and enemies
            if !bullet.is_enemy {
                for enemy in &mut enemies {
                    if (enemy.pos - bullet.pos).length() < player_size / 2.0 { // Rough collision radius
                        enemy.health -= 1; // Decrease enemy health
                        if enemy.health <= 0 {
                            return false; // Enemy dies when health reaches 0
                        }
                    }
                }
            }

            // Check collision between enemy's bullet and player
            if bullet.is_enemy {
                if (player.pos - bullet.pos).length() < player_size / 2.0 { // Rough collision radius
                    player.health -= 1; // Decrease player health
                    return false; // Player gets hit by enemy bullet
                }
            }

            true
        });

        // --- Update and draw enemies (smaller and with health)
        for enemy in &mut enemies {
            let to_player = player.pos - enemy.pos;
            if to_player.length() > 0.0 {
                enemy.pos += to_player.normalize() * enemy.speed * dt;
            }

            // Draw enemy as a smaller rectangle (or texture, as needed)
            draw_texture_ex(
                &enemy.texture,
                enemy.pos.x,
                enemy.pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(player_size, player_size)), // Same size as player
                    ..Default::default()
                },
            );

            // --- Draw enemy health bar
            let health_percentage = enemy.health as f32 / 3.0; // Assuming 3 health is full
            draw_rectangle(enemy.pos.x, enemy.pos.y - 10.0, player_size, 5.0, GRAY); // Health bar background
            draw_rectangle(
                enemy.pos.x,
                enemy.pos.y - 10.0,
                player_size * health_percentage,
                5.0,
                RED, // Health bar foreground
            );

            // Check if enemy is dead, remove it from the game
            if enemy.health <= 0 {
                continue; // Remove the enemy if it is dead
            }
        }

        // Spawn more enemies when there are only 3 enemies left
        if enemies.len() <= 3 {
            enemies.push(Enemy {
                pos: vec2(rand::gen_range(100.0, screen_width() - 100.0), rand::gen_range(100.0, screen_height() - 100.0)),
                speed: rand::gen_range(50.0, 100.0),
                texture: enemy_texture.clone(),
                health: 3, // Starting health for the new enemy
                last_shot_time: 0.0,
            });
        }

        // --- Draw player (rotated toward mouse) with equal size to enemies
        draw_texture_ex(
            &player.texture,
            player.pos.x,
            player.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: angle,
                dest_size: Some(vec2(player_size, player_size)), // Same size as enemy
                ..Default::default()
            },
        );

        // --- Draw player health bar
        let player_health_percentage = player.health as f32 / 5.0; // Assuming 5 health is full
        draw_rectangle(player.pos.x, player.pos.y - 10.0, player_size, 5.0, GRAY); // Health bar background
        draw_rectangle(
            player.pos.x,
            player.pos.y - 10.0,
            player_size * player_health_percentage,
            5.0,
            GREEN, // Health bar foreground
        );

        // --- Check if player is dead
        if player.health <= 0 {
            break; // End the game when the player dies
        }

        next_frame().await;
    }
}
