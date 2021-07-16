
use js_sys::Math;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::Document;
use web_sys::HtmlCanvasElement;
use web_sys::KeyboardEvent;
use web_sys::Window;
use std::cell::RefCell;
use std::rc::Rc;
use std::f64;
// Called by our JS entry point to run the example

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen]
    fn alert(s: &str);

}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let window :Window = web_sys::window().expect("no global `window` exists");
    let document: Document = window.document().expect("should have a document on window");
    let game_manager = Rc::new(RefCell::new(Game::new( (420,420), 20, ("yellow", "green", "red"))));
    let inner_manager = game_manager.clone();
    game_manager.borrow_mut().start_game_loop(move || {
        inner_manager.borrow().draw_scene();
        inner_manager.borrow_mut().draw_snake();
        inner_manager.borrow_mut().snake.take_a_step();
        inner_manager.borrow_mut().draw_apple();
    });
    let inner_manager = game_manager.clone();
    let a = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        match event.key().as_str(){
            "ArrowUp" => {inner_manager.borrow_mut().snake.set_direction(Direction::UP)},
            "ArrowDown" => {inner_manager.borrow_mut().snake.set_direction(Direction::DOWN);},
            "ArrowLeft" => {inner_manager.borrow_mut().snake.set_direction(Direction::LEFT)},
            "ArrowRight" => {inner_manager.borrow_mut().snake.set_direction(Direction::RIGHT)},
            _ => {},
        }
        
    })as Box<dyn Fn(_)>);
    document.add_event_listener_with_callback("keydown", a.as_ref().unchecked_ref())?;
    a.forget();
    Ok(())
}


struct Game{
    block_size: u32,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    snake: Snake,
    board_color: String,
    apple_color: String,
    game_stat: Rc<RefCell<GameStat>>,
    apple_location: (u32 ,u32),
}

impl Game{
    fn new(size: (u32, u32), block_size: u32, colors: (&str, &str, &str))-> Self{
        let window :Window = web_sys::window().expect("no global `window` exists");
        let document: Document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");
        let canvas = document.create_element("canvas").expect("Canvas oluşturulamadı!");
        let canvas = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        
        canvas.set_width(size.0);
        canvas.set_height(size.1);
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        body.append_child(&canvas).expect("Canvas body'e eklenemedi!");
        let apple_location = (0,0);
        let block_counts = (size.0/block_size, size.1/block_size);
        let game_stat = Rc::new(RefCell::new(GameStat::new()));
        Game{apple_location, game_stat: game_stat.clone() ,canvas, context, snake: Snake::new(colors.0.to_owned(), block_counts, game_stat), block_size, board_color: colors.1.to_owned(), apple_color: colors.2.to_owned()}
    }
    
    fn draw_scene(&self){
        let mut loc = Location{x: 0, y:0};
        let block_counts = (self.canvas.width()/self.block_size, self.canvas.height()/self.block_size);
        for _ in 0..block_counts.0*block_counts.1{
            if loc.x == block_counts.0{
                loc.x = 0;
                loc.y +=1;
            }
            self.draw_block((loc.x, loc.y), &self.board_color);
            loc.x += 1;
        }
    }

    fn draw_snake(&mut self){
        for (i, (_, location)) in self.snake.body.clone().iter().enumerate(){
            self.draw_block((location.x, location.y), &self.snake.color);
            if i == 0{
                if (self.apple_location.0 == location.x)&&(self.apple_location.1 == location.y){
                    self.snake.increase_body();
                    self.apple_location = (0,0);
                }
            }
        }
    }

    fn draw_apple(&mut self){
        if self.apple_location == (0,0){
            let window :Window = web_sys::window().expect("no global `window` exists");
            let block_counts = (self.canvas.width()/self.block_size, self.canvas.height()/self.block_size);
            let location = (Math::floor(Math::random()*block_counts.0 as f64) as u32, Math::floor(Math::random()*block_counts.1 as f64) as u32);
            let mut apple_in_not_snake = true;
            for (_, snake_location) in self.snake.body.iter(){
                let snake_location = (snake_location.x, snake_location.y);
                if location == snake_location{
                    apple_in_not_snake = false;
                }
            }
            if apple_in_not_snake{
                self.apple_location = location;
            }else {
                self.draw_apple();
            }
            
            
        }
        self.draw_block(self.apple_location, self.apple_color.as_str());
    }



    fn draw_block(&self, location: (u32, u32), color: &str){
        self.context.set_fill_style(&JsValue::from_str(color));
        self.context.fill_rect((location.0 * self.block_size) as f64, (location.1 * self.block_size) as f64, (self.block_size - 1) as f64, (self.block_size - 1) as f64);
            
    }

    fn start_game_loop<F: Fn() + 'static>(&mut self, closure: F){
        let window = Rc::new(web_sys::window().expect("no global `window` exists"));
        let inner_window = window.clone();
        let inner_window2 = inner_window.clone();
       
        let f = Rc::new(RefCell::new(None as Option<Closure<dyn Fn()>>));
        let g = f.clone();
        let h = g.clone();
        let b =  Rc::new(RefCell::new(0));
        let a = self.game_stat.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            if b.borrow().clone() > 5{
                closure();
                
                *b.borrow_mut() = 0;
            }else{
                *b.borrow_mut() += 1;
            }
            if a.borrow().stat == true{
                inner_window2.request_animation_frame(h.borrow().as_ref().unwrap().as_ref().unchecked_ref());
            }
        }) as Box<dyn Fn()>));
        self.draw_apple();
        window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }

}


struct Snake{
    body: Vec<(Direction, Location)>,
    color: String,
    block_count: (u32,u32),
    game_stat: Rc<RefCell<GameStat>>
}

impl Snake{
    fn new(color: String, block_count: (u32,u32), game_stat: Rc<RefCell<GameStat>>) -> Self{
        Snake{body: vec![(Direction::UP, Location{x: 10, y: 10})], color, block_count, game_stat}
    }

    fn take_a_step(&mut self){
        let mut previous_body:(Direction, Location) = (Direction::UP, Location{x: 0, y: 0});
        let mut head_location = Location{x: 0, y: 0};
        for (i, (direction, location)) in self.body.iter_mut().enumerate(){
            if i == 0{
                previous_body = (direction.clone(), location.clone());
                match direction{
                    Direction::UP => {
                        if location.y as i32 - 1 >= 0{
                            location.y -= 1
                        }else{
                            location.y = self.block_count.1-1;
                        }
                        
                    },
                    Direction::DOWN => {
                        if location.y + 1 < self.block_count.1{
                            location.y += 1
                        }else{
                            location.y = 0;
                        }
                    },
                    Direction::RIGHT => {
                        if location.x + 1 < self.block_count.0{
                            location.x += 1;
                        }else{
                            location.x = 0;
                        }
                    },
                    Direction::LEFT => {
                        if location.x as i32 - 1 >= 0{
                            location.x -= 1
                        }else{
                            location.x = self.block_count.0-1;
                        }
                    },
                }
                head_location = location.clone();
            }else{
                if &mut head_location != location{
                    let old_body = (direction.clone(), location.clone()); 
                    *direction = previous_body.0;
                    *location = previous_body.1;
                    previous_body = old_body;
                }else{
                    self.game_stat.borrow_mut().stat = false;
                    alert("You Lose!");
                }
                
                
            }

        }
    }

    fn set_direction(&mut self, direction: Direction){
        if let Some(head) = self.body.get_mut(0){
            match (head.0.clone(), &direction){
                (Direction::UP | Direction::DOWN, Direction::LEFT | Direction::RIGHT) => {head.0 = direction;},
                (Direction::LEFT | Direction::RIGHT, Direction::UP | Direction::DOWN) => {head.0 = direction;},
                _ => {}
            }
            
        }
    }

    fn increase_body(&mut self){
        if let Some(head) = self.body.get(0){
            let mut location = head.1.clone();
            let new_head = (head.0.clone(), match head.0{
                Direction::UP => {location.y -= 1; location},
                Direction::DOWN =>{ location.y += 1; location},
                Direction::RIGHT => {location.x += 1; location},
                Direction::LEFT => {location.x -= 1; location},

            });
            self.body.insert(0, new_head);
        }
       
    }
}

struct GameStat{
    stat: bool,
}

impl GameStat{
    fn new() -> Self{
        GameStat{stat: true}
    }

    fn set_stat(&mut self, stat: bool){
        self.stat = stat;
    }
}

#[derive(Debug)]
enum Direction{
    UP,
    DOWN,
    RIGHT,
    LEFT
}

impl Clone for Direction{
    fn clone(&self) -> Self{
        match self{
            Direction::UP => Direction::UP,
            Direction::DOWN =>  Direction::DOWN,
            Direction::RIGHT => Direction::RIGHT,
            Direction::LEFT => Direction::LEFT,
        }
    }
}
#[derive(Debug)]
struct Location{
    x: u32,
    y: u32,
}

impl Clone for Location{
    fn clone(&self) -> Self{
        Location{x: self.x.clone(), y: self.y.clone()}
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        (self.x == other.x) && (self.y == other.y)
    }
}