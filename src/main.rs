use yew::prelude::*;
use std::rc::Rc;
use web_sys::console;
use std::cell::RefCell;
use std::borrow::{BorrowMut, Borrow};
use std::ops::Deref;
use gloo_timers::callback::Interval;
use rand::{thread_rng, Rng};
use std::time::{Duration, SystemTime};

enum GenerateOptions{
    Empty,
    Random
}

impl GenerateOptions {
    fn to_value(&self)->&'static str{
        match self {
            GenerateOptions::Empty => {"empty"}
            GenerateOptions::Random => {"random"}
        }
    }

    fn from_value(value:String)->Self{
        match value.as_str() {
            "random" => {GenerateOptions::Random}
            _ => {GenerateOptions::Empty}
        }
    }
}

enum Msg {
    Generate,
    GenerateRandom,
    ChangeX(Option<usize>),
    ChangeY(Option<usize>),
    ToggleCell(usize, usize),
    Tick(bool),
    ToggleClock,
}

struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    size_x: usize,
    size_y: usize,
    cells:Rc<RefCell<Vec<Vec<bool>>>>,
    interval : Interval,
    threshold: f32,
    active : bool
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| Msg::Tick(false));
        let interval = Interval::new(400,move ||callback.emit(()));
        Self {
            link,
            size_x: 10,
            size_y: 10,
            cells: Rc::new(RefCell::new(vec![])),
            interval,
            threshold: 0.5,
            active:false
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Generate => {
                self.active = false;
                self.cells = Rc::new(RefCell::new(vec![vec![false;self.size_x];self.size_y]));
                true
            }
            Msg::GenerateRandom => {
                let mut rng = thread_rng();
                self.active = false;
                let mut cells :Vec<Vec<bool>>= vec![];
                for i in 0..self.size_y {
                    let mut row: Vec<bool> = vec![];
                    for j in 0..self.size_x {
                        row.push(rng.gen_bool(self.threshold as f64))
                    }
                    cells.push(row);
                }
                self.cells = Rc::new(RefCell::new(cells));
                true
            }
            Msg::ChangeX(val) => {
                if let Some(val) = val {
                    self.size_x = val;
                    true
                }else {
                    false
                }
            }
            Msg::ChangeY(val) => {
                if let Some(val) = val {
                    self.size_y = val;
                    true
                }else {
                    false
                }
            },
            Msg::ToggleCell(x,y)=>{
                if let Some(cell) = RefCell::borrow_mut(&self.cells).get_mut(y).and_then(|row|row.get_mut(x)){
                    *cell = !*cell;
                    true
                }else {
                    false
                }
            },
            Msg::Tick(force)=>{
                if !self.active && !force {
                    return false;
                }
                let start = instant::Instant::now();
                self.tick();
                let duration = instant::Instant::now().duration_since(start);
                console::log_1(&duration.as_millis().into());
                true
            },
            Msg::ToggleClock =>{
                self.active = !self.active;
                false
            }
            _ => {false}
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let test = self.cells.clone();
        html! {
            <body>
                <main class="container">
                    <article>
                        <label>{"Size : "}</label>
                        <div class={"grid"} >
                            <input oninput=self.link.callback(|event:InputData| Msg::ChangeX(event.value.parse().ok())) type={"number"} value={self.size_x.to_string()}/>
                            <input oninput=self.link.callback(|event:InputData| Msg::ChangeY(event.value.parse().ok())) type={"number"} value={self.size_y.to_string()}/>
                        </div>
                        <input type={"range"} min={"0"} max={"1"} step={"0.01"} value={self.threshold.to_string()} />
                        <div class={"grid"} >
                            <button onclick=self.link.callback(|_| Msg::Generate)>{ "Generate" }</button>
                            <button onclick=self.link.callback(|_| Msg::GenerateRandom)>{ "Generate Random" }</button>
                        </div>
                        <div class={"grid"}>
                            <button onclick=self.link.callback(|_| Msg::Tick(true))>{ "Tick" }</button>
                            <button onclick=self.link.callback(|_| Msg::ToggleClock)>{ if self.active {"Stop"} else {"Play"} }</button>
                        </div>
                    </article>
                    <Grid cells={self.cells.clone()} toggle_callback={self.link.callback(|(x,y)|Msg::ToggleCell(x,y))}/>
                </main>
            </body>
        }
    }
}

impl Model{
    fn tick(&mut self){
        let mut new_cells:Vec<Vec<bool>> = vec![];
        {
            let mut cells = RefCell::borrow_mut(&self.cells);
            let height = cells.len();
            let width = cells.get(0).map(|row| row.len()).unwrap_or(0);

            for i in 0..height {
                let mut row: Vec<bool> = vec![];
                for j in 0..width {
                    row.push(Model::tick_cell(&mut *cells, j, i));
                }
                new_cells.push(row);
            }
        }
        self.cells = Rc::new(RefCell::new(new_cells));
    }

    fn tick_cell(cells: &mut Vec<Vec<bool>>, x: usize, y: usize)->bool{
        const ADJACENCY : [(i64,i64); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 1),
            (0, 1),
            (1, 1),
            (-1, 0),
            (1, 0),
        ];
        let xi = x as i64;
        let yi = y as i64;
        let count = ADJACENCY.iter()
            .fold(0 as u8, |accum,(dx,dy)| if Model::get_cell(cells, xi +dx, yi+dy) {accum+1} else { accum });
        if let Some(cell) = cells.get(y).and_then(|row|row.get(x)) {
            (*cell && count>=2 && count<=3) || count == 3
        }else {
            false
        }
    }

    fn get_cell(cells: &Vec<Vec<bool>>, x: i64, y: i64)->bool{
        if x>=0 && y>=0 {
            cells.get(y as usize).and_then(|row| {
                row.get(x as usize)
            }).map(|cell| {
                cell.clone()
            }
            ).unwrap_or(false)
        }
        else { false }
    }
}

#[derive(Properties, Clone)]
struct GridProps{
    cells:Rc<RefCell<Vec<Vec<bool>>>>,
    toggle_callback:Callback<(usize, usize)>
}

enum GridMsg {
    ToggleCell(usize,usize)
}

struct Grid{
    link:ComponentLink<Self>,
    props: GridProps
}

impl Component for Grid{
    type Message = GridMsg;
    type Properties = GridProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self{ link, props }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            GridMsg::ToggleCell(x, y) => { self.props.toggle_callback.emit((x,y))}
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let cells = RefCell::borrow(&self.props.cells);

        html!{
            <div class={"gridb"} data-height={cells.len().to_string()} data-width={cells.get(0).map(|row|row.len()).unwrap_or(0).to_string()}>
                {for cells.iter().enumerate().map(|(i, row)| self.draw_row(row,i))}
            </div>
        }
    }
}

impl Grid{

    fn draw_row(&self,row:&Vec<bool>,y:usize) -> Html{
        html!{
            <div class={"row"}>
            {for row.iter().enumerate().map(|(i, cell)|self.draw_cell(*cell, i, y))}
            </div>
        }
    }

    fn draw_cell(&self, cell:bool, x:usize, y:usize) -> Html{
        html!{
            <div class={if cell {"cell alive"} else {"cell"}}
                onclick={self.link.callback(move |_| GridMsg::ToggleCell(x, y.clone()))}></div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}