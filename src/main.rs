#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate yaml_rust;

pub mod iterators;
pub mod error;

use iterators::IdentifyLast;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{collections::HashMap, fs, io, path::Path};
use std::ops::Range;
use error::{Error, ConvertResult};

const PANDAS_NULL: &str = "NA";

const DIR_PATH: &str = "/home/matteo/projects/revolve/experiments/isaac/data";
const EXPERIMENT_TYPES: &[&str] = &[
    "base_prog",
    "base_rnd",
    "cosit_prog",
    "cosit_rnd",
];

const RUNS: Range<u16> = 1..159;
// const RUNS: &[u16] = &[
    // 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    // 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    // 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
    // 31, 32, 33, 34, 35, 36, 37, 38,
// ];

const BEHAVIOURAL_MEASURES: &[&str] = &[
    "velocity",
    "displacement_velocity",
    "displacement_velocity_hill",
    "head_balance",
    "contacts",
];

const PHENOTYPE_MEASURES: &[&str] = &[
    "branching",
    "branching_modules_count",
    "limbs",
    "extremities",
    "length_of_limbs",
    "extensiveness",
    "coverage",
    "joints",
    "hinge_count",
    "active_hinges_count",
    "brick_count",
    "touch_sensor_count",
    "brick_sensor_count",
    "proportion",
    "width",
    "height",
    "z_depth",
    "absolute_size",
    "sensors",
    "symmetry",
    "vertical_symmetry",
    "height_base_ratio",
    "base_density",
    "bottom_layer",
];

fn open_file_with_headers<P: AsRef<Path>>(path: &P) -> io::Result<fs::File> {
    let file_all_measures = path.as_ref().join("all_measures.tsv");
    let mut file_summary = fs::File::create(file_all_measures)?;

    //WRITE ID + GENERATION + SPECIES_ID + FITNESS + N_PARENTS + PARENT_1 + PARENT_2
    for (last, header) in [
        "robot_id",
        "generation",
        "species",
        "fitness",
        "n_parents",
        "parent1",
        "parent2",
    ]
    .iter()
    .chain(BEHAVIOURAL_MEASURES.iter())
    .chain(PHENOTYPE_MEASURES.iter())
    .identify_last()
    {
        if !last {
            write!(&mut file_summary, "{}\t", header)?;
        } else {
            writeln!(&mut file_summary, "{}", header)?;
        }
    }

    Ok(file_summary)
}

fn load_yaml_to_str<P: AsRef<Path>>(path: &P) -> io::Result<String> {
    Ok(fs::read_to_string(path.as_ref())?
        .replace(":", ": ")
        .replace("None", "null"))
}

fn generate_all_measures<P: AsRef<Path>>(
    run_path: &P,
    id_gen_species_map: &HashMap<u64, Vec<(u64, u64)>>,
    phylogeny: &HashMap<u64, Vec<u64>>,
) -> Result<(), Error> {
    let mut file_summary = open_file_with_headers(run_path).into_error("could not open file_summary")?;

    let phylogeny_filepath = run_path
        .as_ref()
        .join("data_fullevolution")
        .join("filogeny.tsv");
    let mut phylogeny_file =
        fs::File::create(phylogeny_filepath).into_error("Cound not create finlogeny file")?;

    let fitness_filepath = run_path
        .as_ref()
        .join("data_fullevolution")
        .join("fitness.csv");
    let fitness_file =
        io::BufReader::new(fs::File::open(fitness_filepath).into_error("could not open fitness file")?);

    fitness_file
        .lines()
        .map(|line| line.unwrap())
        .map(|line| {
            let mut line_split = line.split(',');
            let robot_id: u64 = line_split.next().unwrap().parse::<u64>().unwrap();
            let fitness: Option<f64> = line_split.next().unwrap().parse::<f64>().ok();
            assert_eq!(None, line_split.next());
            (robot_id, fitness)
        })
        .flat_map(move |(robot_id, fitness)| {
            // replicate line for each gen and species id found
            lazy_static! {
                static ref DEFAULT_VALUE: Vec<(Option<u64>, Option<u64>)> = vec![(None, None)];
            }
            id_gen_species_map
                .get(&robot_id)
                .map(move |slice| {
                    slice
                        .iter()
                        .map(|(generation, species)| (Some(*generation), Some(*species)))
                        .collect::<Vec<_>>()
                })
                .as_ref()
                .unwrap_or(&DEFAULT_VALUE)
                .iter()
                .map(move |(generation, species)| {
                    (robot_id, generation.clone(), species.clone(), fitness)
                })
                .collect::<Vec<_>>()
        })
        .map(
            |(robot_id, generation, species_id, fitness): (
                u64,
                Option<u64>,
                Option<u64>,
                Option<f64>,
            )| {
                // add phylogeny data
                let parents: &Vec<u64> = phylogeny.get(&robot_id).unwrap();
                let n_parents = parents.len();
                let parent1 = parents.get(0);
                let parent2 = parents.get(1);
                (
                    robot_id, generation, species_id, fitness, n_parents, parent1, parent2,
                )
            },
        )
        .for_each(
            |(robot_id, generation, species_id, fitness, n_parents, parent1, parent2)| {
                // WRITE ID + GENERATION + SPECIES_ID + FITNESS + N_PARENTS + PARENT_1 + PARENT_2
                let fitness = fitness.unwrap_or(0.0);

                let parent1 = parent1.map(|id| id.to_string());
                let parent1 = parent1
                    .as_ref()
                    .map(|string| string.as_str())
                    .unwrap_or(PANDAS_NULL);

                let parent2 = parent2.map(|id| id.to_string());
                let parent2 = parent2
                    .as_ref()
                    .map(|string| string.as_str())
                    .unwrap_or(PANDAS_NULL);

                writeln!(
                    &mut phylogeny_file,
                    "{}\t{}\t{}\t{}",
                    robot_id, n_parents, parent1, parent2
                )
                .unwrap();

                let generation = generation.map(|id| id.to_string());
                let generation = generation
                    .as_ref()
                    .map(|string| string.as_str())
                    .unwrap_or(PANDAS_NULL);

                let species_id = species_id.map(|id| id.to_string());
                let species_id = species_id
                    .as_ref()
                    .map(|string| string.as_str())
                    .unwrap_or(PANDAS_NULL);

                write!(
                    &mut file_summary,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t",
                    robot_id, generation, species_id, fitness, n_parents, parent1, parent2
                )
                .unwrap();

                // BEHAVIOURAL MEASURES -------------------------------------------
                let behaviour_filename = run_path
                    .as_ref()
                    .join("data_fullevolution")
                    .join("descriptors")
                    .join("behavioural")
                    .join(format!("behavior_desc_{}.txt", robot_id));

                if let Ok(behaviour_file) = fs::File::open(behaviour_filename) {
                    let mut file_reader = io::BufReader::new(behaviour_file).lines().peekable();
                    let first_line = file_reader.peek().unwrap().as_ref().unwrap();
                    if first_line == "None" {
                        for _ in BEHAVIOURAL_MEASURES {
                            write!(&mut file_summary, "{}\t", PANDAS_NULL).unwrap();
                        }
                    } else {
                        let behavior_measures = file_reader
                            .map(|line| {
                                let line = line.unwrap();
                                let mut split = line.trim().split(' ');
                                let measure = split.next().unwrap().to_string();
                                let value = split.next().unwrap().parse::<f64>().ok();
                                assert_eq!(None, split.next());
                                (measure, value)
                            })
                            .collect::<HashMap<String, Option<f64>>>();

                        for measure in BEHAVIOURAL_MEASURES {
                            let value = behavior_measures
                                .get(*measure)
                                .unwrap()
                                .map(|v| v.to_string());
                            let value = value.as_ref().map(|v| v.as_str()).unwrap_or(PANDAS_NULL);
                            write!(&mut file_summary, "{}\t", value).unwrap();
                        }
                    }
                } else {
                    for _ in BEHAVIOURAL_MEASURES {
                        write!(&mut file_summary, "{}\t", PANDAS_NULL).unwrap();
                    }
                }

                // PHENOTYPES MEASURES --------------------------------------------
                let phenotype_filename = run_path
                    .as_ref()
                    .join("data_fullevolution")
                    .join("descriptors")
                    .join(format!("phenotype_desc_{}.txt", robot_id));
                if let Ok(phenotype_file) = fs::File::open(phenotype_filename) {
                    let phenotype_measures = io::BufReader::new(phenotype_file)
                        .lines()
                        .map(|line| {
                            let line = line.unwrap();
                            let mut split = line.trim().split(' ');
                            let measure = split.next().unwrap().to_string();
                            let value = split.next().unwrap().parse::<f64>().ok();
                            assert_eq!(None, split.next());
                            (measure, value)
                        })
                        .collect::<HashMap<String, Option<f64>>>();

                    for (last, measure) in PHENOTYPE_MEASURES.iter().identify_last() {
                        let value = phenotype_measures
                            .get(*measure)
                            .unwrap()
                            .map(|v| v.to_string());
                        let value = value.as_ref().map(|v| v.as_str()).unwrap_or(PANDAS_NULL);
                        if last {
                            writeln!(&mut file_summary, "{}", value).unwrap();
                        } else {
                            write!(&mut file_summary, "{}\t", value).unwrap();
                        }
                    }
                } else {
                    for (last, _) in PHENOTYPE_MEASURES.iter().identify_last() {
                        if last {
                            writeln!(&mut file_summary, "{}", PANDAS_NULL).unwrap();
                        } else {
                            write!(&mut file_summary, "{}\t", PANDAS_NULL).unwrap();
                        }
                    }
                }

                writeln!(&mut file_summary).unwrap();
            },
        );

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SpeciesAge {
    evaluations: u64,
    generations: u64,
    no_improvements: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Species {
    id: u64,
    age: SpeciesAge,
    individuals_ids: Vec<u64>,
}

impl Species {
    pub fn parse_from_file<P: AsRef<Path>>(path: &P) -> Result<Self, Error> {
        let species_str = load_yaml_to_str(path)
            .into_error("read yaml file failed")?;
        let species: Self = serde_yaml::from_str(&species_str)
            .into_error("parse yaml file failed")?;
        Ok(species)
    }
}

fn generate_shaphot_ids<P: AsRef<Path>>(run_path: &P) -> HashMap<u64, Vec<(u64, u64)>> {
    // Generation, robot_id
    println!(
        "Generating snaphost_ids for {}",
        run_path.as_ref().display()
    );

    let ids_filepath = run_path.as_ref().join("snapshots_ids.tsv");
    let mut ids_file = fs::File::create(ids_filepath).expect("could not create snapshot_ids file");
    write!(&mut ids_file, "generation\trobot_id\tspecies_id\n").unwrap();
    lazy_static! {
        static ref GENERATION_REGEX: Regex = Regex::new(r"^generation_(\d+)$").unwrap();
    }

    let generations_path = run_path.as_ref().join("generations");
    //TODO return optional species
    let mut generated_ids_map: HashMap<u64, Vec<(u64, u64)>> = HashMap::new();

    for path in fs::read_dir(&generations_path).unwrap() {
        let path = path.unwrap();
        let filename = path.file_name();
        let filename = filename.to_str().unwrap_or("");
        if let Some(gen_num_str) = GENERATION_REGEX.captures(filename) {
            let captured_str = &gen_num_str[1]; // 0 is the whole string, 1 is the first match
            let gen_num = captured_str.parse::<u64>().unwrap();

            let ids_filename = generations_path.join(filename).join("identifiers.txt");
            let file = fs::File::open(ids_filename)
                .expect("Could not open identifiers.txt file");

            for line in io::BufReader::new(file).lines() {
                let individual_id = line.unwrap().parse::<u64>().unwrap();
                generated_ids_map
                    .entry(individual_id)
                    .or_default()
                    .push((gen_num, 0));
                write!(
                    &mut ids_file,
                    "{}\t{}\t{}\n",
                    gen_num, individual_id, 0
                )
                    .unwrap();
            }
        } else {
            println!("unread folder {}", filename);
        }
    }

    generated_ids_map
}

fn generate_shaphot_ids_generations_species<P: AsRef<Path>>(run_path: &P) -> HashMap<u64, Vec<(u64, u64)>> {
    // Generation, robot_id
    println!(
        "Generating snaphost_ids for {}",
        run_path.as_ref().display()
    );

    let ids_filepath = run_path.as_ref().join("snapshots_ids.tsv");
    let mut ids_file = fs::File::create(ids_filepath).expect("could not create snapshot_ids file");
    write!(&mut ids_file, "generation\trobot_id\tspecies_id\n").unwrap();
    lazy_static! {
        //static ref POP_FOLDER_REGEX: Regex = Regex::new(r"^selectedpop_(\d+)$").unwrap();
        static ref GENERATION_REGEX: Regex = Regex::new(r"^generation_(\d+)$").unwrap();
        static ref SPECIES_FILE_REGEX: Regex = Regex::new(r"^species_(\d+).yaml$").unwrap();
    }

    let generations_path = run_path.as_ref().join("generations");
    let mut generated_ids_map: HashMap<u64, Vec<(u64, u64)>> = HashMap::new();

    for path in fs::read_dir(&generations_path).unwrap() {
        let path = path.unwrap();
        let filename = path.file_name();
        let filename = filename.to_str().unwrap_or("");
        if let Some(gen_num_str) = GENERATION_REGEX.captures(filename) {
            let captured_str = &gen_num_str[1]; // 0 is the whole string, 1 is the first match
            let gen_num = captured_str.parse::<u64>().unwrap();

            let species_path = generations_path.join(filename);
            for species_file in fs::read_dir(species_path).unwrap() {
                let species_file = species_file.unwrap();
                let species_filename = species_file.file_name();
                let species_filename = species_filename.to_str().unwrap_or("");
                if let Some(species_filename_regex_match) =
                    SPECIES_FILE_REGEX.captures(species_filename)
                {
                    let species_num_from_filename =
                        species_filename_regex_match[1].parse::<u64>().unwrap();

                    let species = Species::parse_from_file(&species_file.path())
                        .expect("could not read phylogeny file for individual");

                    assert_eq!(species.id, species_num_from_filename);
                    for individual_id in species.individuals_ids {
                        generated_ids_map
                            .entry(individual_id)
                            .or_default()
                            .push((gen_num, species.id));
                        write!(
                            &mut ids_file,
                            "{}\t{}\t{}\n",
                            gen_num, individual_id, species.id
                        )
                        .unwrap();
                    }
                }
            }
        } else {
            println!("unread folder {}", filename);
        }
    }

    generated_ids_map
}

fn load_phylogeny<P: AsRef<Path>>(path: &P) -> Result<HashMap<u64, Vec<u64>>, Error> {
    let phylogeny_folder = path.as_ref().join("data_fullevolution").join("phylogeny");

    let dir_reader = fs::read_dir(&phylogeny_folder)
        .into_error(format!("Could not open phylogeny folder ({})", phylogeny_folder.display()))?;

    dir_reader.filter_map(|phylogeny_file| {
            let phylogeny_file = phylogeny_file.unwrap();
            let filename = phylogeny_file.file_name();
            let filename = filename.to_str().unwrap_or("");
            lazy_static! {
                static ref PHYLOGENY_FILE_REGEX: Regex =
                    Regex::new(r"^parents_(\d+).yaml$").unwrap();
            }
            if let Some(robot_id) = PHYLOGENY_FILE_REGEX.captures(filename) {
                robot_id[1]
                    .parse::<u64>()
                    .map(|robot_id| (robot_id, phylogeny_file))
                    .ok()
            } else {
                None
            }
        })
        .map(|(robot_id, phylogeny_file)| {
            let mut robot_phylogeny_str = load_yaml_to_str(&phylogeny_file.path())
                .expect("could not read phylogeny file for individual");
            robot_phylogeny_str += "\n";
            use yaml_rust::{Yaml, YamlLoader};
            let parents = if robot_phylogeny_str == "parents: null\n" {
                Vec::<u64>::new()
            } else {
                let robot_phylogeny = YamlLoader::load_from_str(&robot_phylogeny_str)
                    .into_error("Error loading yaml from string")?;
                let parents: Vec<u64> = match &robot_phylogeny[0]["parents"] {
                    Yaml::Array(array) => array
                        .iter()
                        .map(|node| node.as_i64().unwrap() as u64)
                        .collect(),
                    Yaml::Null => Vec::new(),
                    Yaml::Real(_) => panic!("phylogeny parents yaml parse error: Real"),
                    Yaml::Integer(single_parent) => vec![*single_parent as u64],
                    Yaml::String(text) => text.split(",").map(|v| v.parse().unwrap()).collect(),
                    Yaml::Boolean(_) => panic!("phylogeny parents yaml parse error: Boolean"),
                    Yaml::Hash(_) => panic!("phylogeny parents yaml parse error: Hash"),
                    Yaml::Alias(_) => panic!("phylogeny parents yaml parse error: Alias"),
                    Yaml::BadValue => panic!("phylogeny parents yaml parse error: BadValue"),
                };
                parents
            };

            Ok((robot_id, parents))
        })
        .collect()
}

fn analyze(exp: &str, run: u16) -> Result<(), Error> {
    print!("Consilidating {}, run {} ... ", exp, run);
    let run_path = Path::new(DIR_PATH).join(exp).join(run.to_string());
    let phylogeny = load_phylogeny(&run_path)?;
    let id_gen_species_map = generate_shaphot_ids(&run_path);
    generate_all_measures(&run_path, &id_gen_species_map, &phylogeny)
}

fn main() {
    for exp in EXPERIMENT_TYPES {
        for run in RUNS {
            let r = analyze(exp, run);
            if r.is_err() {
                println!("{}:{} failed because {:?}", exp, run, r);
            }
        }
    }
}
