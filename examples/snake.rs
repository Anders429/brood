use brood::{entity, entity::EntityIdentifier, query, registry, result, views, World};
use crossterm::{
    cursor, event, style,
    style::{Color, ContentStyle, StyledContent, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use fps_clock::FpsClock;
use rand::random;
use std::{
    io::{stdout, Stdout, Write},
    iter,
    time::Duration,
};

const WIDTH: u16 = 64;
const HEIGHT: u16 = 48;

#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: u16,
    y: u16,
}

#[derive(Clone, Debug)]
struct Length(u8);

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug)]
struct Lifetime(u8);

#[derive(Debug)]
struct Snake;

struct Body;

struct Food;

type Registry = registry!(Position, Color, Length, Direction, Lifetime, Snake, Body, Food);

fn change_snake_direction(world: &mut World<Registry>, new_direction: Direction) {
    for result!(direction) in world.query::<views!(&mut Direction), query::Has<Snake>>() {
        match (*direction, new_direction) {
            (Direction::Up | Direction::Down, Direction::Left | Direction::Right)
            | (Direction::Left | Direction::Right, Direction::Up | Direction::Down) => {
                *direction = new_direction;
            }
            _ => {}
        }
    }
}

fn read_input(world: &mut World<Registry>) -> anyhow::Result<bool> {
    if event::poll(Duration::from_millis(1))? {
        match event::read()? {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Esc | event::KeyCode::Enter => {
                    return Ok(false);
                }
                event::KeyCode::Up => {
                    change_snake_direction(world, Direction::Up);
                }
                event::KeyCode::Right => {
                    change_snake_direction(world, Direction::Right);
                }
                event::KeyCode::Down => {
                    change_snake_direction(world, Direction::Down);
                }
                event::KeyCode::Left => {
                    change_snake_direction(world, Direction::Left);
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(true)
}

fn generate_snake_parts(world: &mut World<Registry>) {
    let mut new_parts = Vec::new();
    for result!(position, length) in world.query::<views!(&Position, &Length), query::None>() {
        new_parts.push((position.clone(), length.clone()));
    }
    for (position, length) in new_parts.into_iter() {
        world.push(entity!(
            position,
            Lifetime(length.0 - 1),
            Color::DarkBlue,
            Body
        ));
    }
}

fn r#move(world: &mut World<Registry>) {
    for result!(position, direction) in
        world.query::<views!(&mut Position, &Direction), query::None>()
    {
        match direction {
            Direction::Up => {
                //position.y -= 1;
                position.y = position.y.saturating_sub(1);
            }
            Direction::Right => {
                // position.x += 1;
                position.x = position.x.saturating_add(1);
            }
            Direction::Down => {
                // position.y += 1;
                position.y = position.y.saturating_add(1);
            }
            Direction::Left => {
                // position.x -= 1;
                position.x = position.x.saturating_sub(1);
            }
        }
    }
}

fn decrement_snake_piece_lifetimes(world: &mut World<Registry>) {
    let mut deleting = Vec::new();
    for result!(entity_identifier, lifetime) in
        world.query::<views!(EntityIdentifier, &mut Lifetime), query::None>()
    {
        if lifetime.0 == 0 {
            deleting.push(entity_identifier);
        } else {
            lifetime.0 -= 1;
        }
    }

    for entity_identifier in deleting {
        world.remove(entity_identifier);
    }
}

fn create_food(world: &mut World<Registry>) {
    world.push(entity!(
        Position {
            x: random::<u16>() % 20,
            y: random::<u16>() % 20,
        },
        Color::Red,
        Food,
    ));
}

fn consume_food(world: &mut World<Registry>) -> bool {
    let mut heads = Vec::new();
    for result!(head_identifier, position, length) in
        world.query::<views!(EntityIdentifier, &Position, &Length), query::Has<Snake>>()
    {
        heads.push((head_identifier, position.clone(), length.clone()));
    }
    let mut eaten = Vec::new();
    for result!(food_identifier, position) in
        world.query::<views!(EntityIdentifier, &Position), query::Has<Food>>()
    {
        for (head_identifier, head_position, length) in &heads {
            if head_position == position {
                eaten.push((*head_identifier, length, food_identifier));
            }
        }
    }
    for (head_identifier, length, food_identifier) in &eaten {
        world.remove(*food_identifier);
        world
            .entry(*head_identifier)
            .unwrap()
            .add(Length(length.0 + 1));
        create_food(world);
    }
    !eaten.is_empty()
}

fn draw(world: &mut World<Registry>, stdout: &mut Stdout) -> anyhow::Result<()> {
    stdout.execute(cursor::RestorePosition)?;
    let (_, top_line) = cursor::position()?;
    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    // Draw border.
    stdout.queue(style::PrintStyledContent("╔".yellow()))?;
    stdout.queue(style::PrintStyledContent(
        iter::repeat("═").take(40).collect::<String>().yellow(),
    ))?;
    stdout.queue(style::PrintStyledContent("╗".yellow()))?;
    for _ in 0..20 {
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.queue(style::PrintStyledContent("║".yellow()))?;
        stdout.queue(cursor::MoveRight(40))?;
        stdout.queue(style::PrintStyledContent("║".yellow()))?;
    }
    stdout.queue(cursor::MoveToNextLine(1))?;
    stdout.queue(style::PrintStyledContent("╚".yellow()))?;
    stdout.queue(style::PrintStyledContent(
        iter::repeat("═").take(40).collect::<String>().yellow(),
    ))?;
    stdout.queue(style::PrintStyledContent("╝".yellow()))?;

    // Draw entities.
    for result!(position, color) in world.query::<views!(&Position, &Color), query::None>() {
        let mut content_style = ContentStyle::new();
        content_style.foreground_color = Some(*color);
        let styled_content = StyledContent::new(content_style, "██");

        stdout
            .queue(cursor::MoveTo(
                position.x * 2 + 1,
                top_line + position.y + 1,
            ))?
            .queue(style::PrintStyledContent(styled_content))?;
    }

    // Draw score.

    stdout.flush()?;
    Ok(())
}

fn run(stdout: &mut Stdout) -> anyhow::Result<()> {
    print!("{}", iter::repeat('\n').take(23).collect::<String>());
    terminal::enable_raw_mode()?;
    stdout.execute(cursor::MoveUp(23))?;
    stdout.execute(cursor::SavePosition)?;
    stdout.execute(cursor::Hide)?;

    let mut world = World::<Registry>::new();
    // Create player.
    world.push(entity!(
        Position { x: 5, y: 5 },
        Color::DarkBlue,
        Length(2),
        Direction::Up,
        Snake,
    ));
    create_food(&mut world);

    let mut fps = FpsClock::new(20);

    loop {
        if !read_input(&mut world)? {
            break;
        }

        generate_snake_parts(&mut world);
        r#move(&mut world);
        if !consume_food(&mut world) {
            decrement_snake_piece_lifetimes(&mut world);
        }
        draw(&mut world, stdout)?;

        fps.tick();
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut stdout = stdout();

    let result = run(&mut stdout);

    terminal::disable_raw_mode()?;
    stdout.execute(cursor::RestorePosition)?;
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
    stdout.execute(cursor::Show)?;

    result
}
