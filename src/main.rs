// Namespace imports to make the code less verbose.
use std::{any, env, fs, path::Path, str::FromStr, time::Duration};
use sdl2::{event::Event, EventPump, pixels::Color, rect::Point, render::Canvas, video::Window};

// Declare and define a function which panics (prints error and exits) if the given path does not resolve to anything.
fn ensure_exists(path: &Path) {
    if !path.exists() {
        panic!("`{}` does not exist.", path.display());
    }
}

// Function which panics if the given path does not resolve or is not a directory.
fn ensure_dir(path: &Path) {
    ensure_exists(path);
    if !path.is_dir() {
        panic!("`{}` is not a directory.", path.display());
    }
}

// Function which panics if the given path does not resolve or is not a file.
fn ensure_file(path: &Path) {
    ensure_exists(path);
    if !path.is_file() {
        panic!("`{}` is not a file.", path.display());
    }
}

// Function which takes a file path and reads and returns all of the text lines from it.
fn read_lines(path: &Path) -> Vec<String> {
    // Read the file as a string, split it by line, construct a String from each fragment, tie it all into a vector, and return it.
    fs::read_to_string(path).unwrap().lines().map(String::from).collect()
}

// Templated function which parses an "x, y" string into a tuple of x and y parsed into the requested type.
fn get_pair<T: FromStr>(string: &str, list_error: &str, purpose_string: &str) -> (T, T) {
    // Split the "x, y" into "x", "y".
    let pair_strings: Vec<_> = string.split(", ").collect();
    if pair_strings.len() != 2 {
        panic!("{}", list_error);
    }
    // The return tuple.
    (
        // Parse "x" and "y" into the requested type, if failing giving a panic message which includes the name of the type.
        pair_strings[0].parse::<T>().unwrap_or_else(|_| panic!("The x {} \"{}\" is not a valid {}.", purpose_string, pair_strings[0], any::type_name::<T>())),
        pair_strings[1].parse::<T>().unwrap_or_else(|_| panic!("The y {} \"{}\" is not a valid {}.", purpose_string, pair_strings[1], any::type_name::<T>()))
    )
}

// Function which parses a file with our metadata structure into the metadata pairs: size (unsigned) and offset (signed).
fn get_metadata(path: &Path) -> ((u32, u32), (i32, i32)) {
    let info = read_lines(path);
    if info.len() != 2 {
        panic!("`{}` does not have exactly two lines.", path.display());
    }

    (
        // Calls the templated function with the generic type set to u32, then i32, for each pair.
        get_pair::<u32>(info[0].as_str(), format!("The size parameter (first line in `{}`) is not a pair \"x, y\".", path.display()).as_str(), "size"),
        get_pair::<i32>(info[1].as_str(), format!("The offset parameter (second line in `{}`) is not a pair \"x, y\".", path.display()).as_str(), "offset")
    )
}

// Function which parses an "x, y" string of floats and an offset pair into an integer point with the offset applied, with the Y coordinate flipped prior for graphics reasons.
fn get_polygon_point(string: &str, offset_pair: (i32, i32)) -> Point {
    // Get the pair as a 32-bit floating-point number ("float" in Java, etc.).
    let float_pair = get_pair::<f32>(string, format!("A polygon contains an invalid pair \"{}\"", string).as_str(), "coordinate");
    // Construct the return point, flipping the Y coordinate and adding the offset coordinates.
    Point::new(float_pair.0 as i32 + offset_pair.0, -float_pair.1 as i32 + offset_pair.1)
}

// Function which parses a string of "(x1, y1), (x2, y2), [...], (xf, yf)" into a list of points.
fn get_polygon(string: &String, offset_pair: (i32, i32)) -> Vec<Point> {
    // Split the "(x1, y1), (x2, y2), ..., (xf, yf)" into "(x1, y1", "x2, y2", ..., "xf, yf)".
    let mut point_strings: Vec<_> = string.split("), (").collect();

    // Fix the first and last strings by removing the parentheses.
    let last_index = point_strings.len() - 1;
    let fixed_first = point_strings[0].replace("(", "");
    let fixed_last = point_strings[last_index].replace(")", "");
    point_strings[0] = fixed_first.as_str();
    point_strings[last_index] = fixed_last.as_str();

    // Take the list, process it with our polygon point function, tie it together into a vector, and return it.
    // The very important thing to note here is that this and the previous iterator for point_strings do indeed process in order, otherwise our polygon would be jumbled.
    point_strings.iter().map(|x| get_polygon_point(x, offset_pair)).collect()
}

// Function which parses a file with a list of polygons' coordinates into a list of lists of points.
fn get_polygons(path: &Path, offset_pair: (i32, i32)) -> Vec<Vec<Point>> {
    // Take the list of point list strings, process it with our polygon function, tie it together into a vector, and return it.
    read_lines(path).iter().map(|x| get_polygon(x, offset_pair)).collect()
}

// Function which parses a directory containing a metadata file and a list of polygons into the list of polygons and the dimensions.
fn read_files(directory: String) -> (Vec<Vec<Point>>, (u32, u32)) {
    // Construct path object from input directory.
    let dir_path = Path::new(directory.as_str());
    // Derive path object from directory path object and filenames.
    let info_path = dir_path.join("collision_info.txt");
    let polygons_path = dir_path.join("polygons.txt");

    // Make sure our path actually exist and are what they are supposed to be in terms of paths vs. files.
    ensure_dir(dir_path);
    ensure_file(info_path.as_path());
    ensure_file(polygons_path.as_path());

    // Initialize multiple variables at once to the members of the tuple returned by the function.
    let (size_pair, offset_pair) = get_metadata(info_path.as_path());

    // Return tuple from our list of lists of points and the parsed dimensions.
    (get_polygons(polygons_path.as_path(), offset_pair), size_pair)
}

// Function which initializes the SDL2 library and returns a tuple with handles to a canvas and to an event queue.
fn init_sdl2(dimensions: (u32, u32)) -> (Canvas<Window>, EventPump) {
    // Initialize and create a handle for the SDL2 library, and panic (print to stderr and exit) if it fails.
    let sdl_context = sdl2::init().unwrap();

    (
        // Create a window and get a handle to its Canvas, which is used for drawing simple graphics like rectangles.
        sdl_context.video().unwrap().window("Level Preview", dimensions.0, dimensions.1).position_centered().build().unwrap().into_canvas().build().unwrap(),

        // Get a handle to SDL2's event queue, which handles stuff like keystrokes and window controls.
        sdl_context.event_pump().unwrap()
    )
}

// Defines the entrypoint function.
fn main() {
    // Collects the equivalent of C's argc and argv into a list of arguments.
    let args: Vec<_> = env::args().collect();
    // If the user did not provide the right number of arguments...
    if args.len() != 2 {
        // ... tell the user how to use the program and terminate.
        println!("Usage: {} <path>", args[0]);
    }
    // Otherwise, proceed with the program.
    else {
        // Feed the directory supplied by the user into our parse functions.
        let (polygon_list, dimensions) = read_files(args[1].clone());

        // Get our canvas and event queue handles.
        let (mut canvas, mut event_pump) = init_sdl2(dimensions);

        // Set what will be used as a frame duration to 1 billion microseconds integer divided by 60, which means targeting 60 FPS.
        let frame_duration = Duration::new(0, 1_000_000_000 / 60);

        // Tell the canvas draw code that the following draw or fill commands should be done with the built-in color red.
        canvas.set_draw_color(Color::RED);

        // For every list of points in our polygon list...
        for polygon in polygon_list {
            // ... draw a path through each point...
            canvas.draw_lines(polygon.as_slice()).unwrap();
            // ... and draw a line from the end to the beginning.
            canvas.draw_line(polygon[0], polygon[polygon.len() - 1]).unwrap();
        }

        // SDL2 does multi-buffering, and this is how we instruct the library to show the framebuffer we've been drawing on.
        canvas.present();

        // This loop is needed only because the program terminates when main returns, and the window closes when the program terminates.
        // The loop is labelled so it can be broken from an inner loop.
        'outer: loop {
            // Loop over every new event since the last check.
            for event in event_pump.poll_iter() {
                // Switch over every type of event.
                match event {
                    // Quit event is when the user clicks the window's close button or uses any similar polite OS close feature.
                    // Jumps out of the outer loop, which in this case is at the end of the program.
                    // If any cleanup code was needed, it could come after the outer loop.
                    Event::Quit { .. } => break 'outer,
                    // We do not care about any other events.
                    _ => {}
                }
            }

            // Block this thread for our specified duration (which we set such that the program runs at about 60 FPS).
            std::thread::sleep(frame_duration);
        }
    }
}
