use contrast::contrast;
use css_colors::{percent, Color};
use image::imageops;
use image::imageops::FilterType;
use rgb::RGB8;
use sciolyff::interpreter::{html::HTMLOptions, Interpreter};
use std::{
    collections::HashMap,
    default::Default,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
};

fn main() -> io::Result<()> {
    let tournaments = get_tournament_info()?;
    write_results_pages(&tournaments)?;

    Ok(())
}

struct Tournament {
    interpreter: Interpreter,
    source_file_name: OsString,
    logo_path: PathBuf,
    theme_color: String,
}

fn get_tournament_info() -> io::Result<Vec<Tournament>> {
    let mut tournaments = Vec::new();

    let entries = fs::read_dir("results")?;
    let logo_info = get_logo_info()?;
    for entry in entries {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let yaml = fs::read_to_string(&path)?;
        let interpreter = Interpreter::from_yaml(&yaml);
        let source_file_name = path.file_name().unwrap().to_os_string();
        let logo_path = get_logo_path(&source_file_name, &logo_info)?;
        let theme_color = get_theme_color(&logo_path);

        tournaments.push(Tournament {
            interpreter,
            source_file_name,
            logo_path,
            theme_color,
        });
    }

    Ok(tournaments)
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct Logo {
    division: Option<String>,
    minimum_year: u32,
    path: PathBuf,
}

fn get_logo_info() -> io::Result<HashMap<String, Vec<Logo>>> {
    let mut logo_info = HashMap::new();

    let entries = fs::read_dir("public/results/logos")?;
    for entry in entries {
        let path = entry?.path();
        let file_name = path
            .file_stem()
            .unwrap()
            .to_str()
            .expect("logo file name must be valid Unicode");
        let splits = file_name.split('_').collect::<Vec<_>>();

        let minimum_year: u32 = splits[0].parse().unwrap_or(0);
        let start_index = if minimum_year == 0 { 0 } else { 1 };
        let (division, tournament_name) = if splits[splits.len() - 1].len() == 1
        {
            (
                Some(splits[splits.len() - 1].to_string()),
                splits[start_index..(splits.len() - 1)].join("_"),
            )
        } else {
            (
                None,
                splits[start_index..splits.len()].join("_").to_string(),
            )
        };

        let entry = logo_info.entry(tournament_name).or_insert_with(Vec::new);
        entry.push(Logo {
            division,
            minimum_year,
            path,
        });
    }

    for logos in logo_info.values_mut() {
        logos.sort();
        logos.reverse();
    }

    Ok(logo_info)
}

fn get_logo_path(
    source_file_name: &OsStr,
    logo_info: &HashMap<String, Vec<Logo>>,
) -> io::Result<PathBuf> {
    let default_logo_path = PathBuf::from("public/results/logos/default.png");

    let source_file_str = source_file_name
        .to_str()
        .expect("results file name must be valid Unicode");
    let year: u32 = source_file_str.splitn(2, '-').collect::<Vec<_>>()[0]
        .parse()
        .expect("results file name must start with a year");
    let splits = source_file_str.splitn(2, '_').collect::<Vec<_>>()[1]
        .rsplitn(2, '_')
        .collect::<Vec<_>>();

    let division = splits[0].splitn(2, '.').collect::<Vec<_>>()[0];
    let tournament_name = splits[1];

    let logo_path = match logo_info.get(tournament_name) {
        Some(logos) => {
            match logos.iter().find(|logo| {
                (logo.division.is_none()
                    || logo.division.as_ref().unwrap() == division)
                    && logo.minimum_year <= year
            }) {
                Some(logo) => logo.path.clone(),
                None => default_logo_path,
            }
        }
        None => default_logo_path,
    };

    Ok(logo_path)
}

fn get_theme_color(logo_path: &Path) -> String {
    let pixel = match image::open(logo_path) {
        Ok(image) => {
            imageops::resize(&image.into_rgb(), 1, 1, FilterType::Triangle)
                .into_raw()
        }
        // SVGs
        Err(_) => vec![48, 48, 48],
    };

    let mut color = css_colors::rgb(pixel[0], pixel[1], pixel[2]);
    let text_color = RGB8::new(255, 255, 255);

    while contrast::<_, f32>(
        RGB8::new(color.r.as_u8(), color.g.as_u8(), color.b.as_u8()),
        text_color,
    ) < 7.0
    {
        color = color.darken(percent(1));
    }

    color.to_css()
}

fn write_results_pages(tournaments: &[Tournament]) -> io::Result<()> {
    fs::create_dir_all("public/results")?;

    for tournament in tournaments {
        let mut path = PathBuf::from("public/results");
        path.push(&tournament.source_file_name);
        path.set_extension("html");

        fs::write(
            path,
            tournament.interpreter.to_html(&HTMLOptions {
                color: tournament.theme_color.clone(),
                ..Default::default()
            }),
        )?;
    }

    Ok(())
}
