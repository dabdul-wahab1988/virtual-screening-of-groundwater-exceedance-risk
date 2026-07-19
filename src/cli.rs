use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "gvs", version = "0.1.0")]
#[command(about = "Groundwater Virtual Screening — leakage-controlled ML for the Pru Basin, Ghana")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Validate all input files and report readiness
    ValidateInputs {
        #[arg(long, help = "Path to raw CSV dataset")]
        data: String,
        #[arg(long, help = "Path to manuscript outline text file")]
        outline: String,
        #[arg(long, help = "Path to thresholds YAML")]
        thresholds: String,
        #[arg(long, help = "Path to leakage-rules YAML")]
        leakage: String,
        #[arg(long, help = "Path to runtime config YAML")]
        config: String,
    },

    /// Initialise empty SQLite database with full schema
    InitDb {
        #[arg(long, help = "Path for the SQLite database")]
        db: String,
    },

    /// Ingest all inputs into the SQLite database
    IngestInputs {
        #[arg(long)]
        data: String,
        #[arg(long)]
        outline: String,
        #[arg(long)]
        thresholds: String,
        #[arg(long)]
        leakage: String,
        #[arg(long)]
        config: String,
        #[arg(long)]
        db: String,
    },

    /// Audit leakage rules and store decisions in SQLite
    AuditLeakage {
        #[arg(long)]
        db: String,
    },

    /// Run ML pipeline for a single target / tier / CV mode
    RunTarget {
        #[arg(long, help = "Target contaminant (Na|Cl|TDS|B|F|NO3)")]
        target: String,
        #[arg(long, help = "Predictor tier (Tier1_Field|Tier2_Reduced|Tier3_Full)")]
        tier: String,
        #[arg(
            long,
            default_value = "Stratified_Nested_CV",
            help = "CV mode (Stratified_Nested_CV|Spatial_Group_CV)"
        )]
        cv_mode: String,
        #[arg(long)]
        db: String,
    },

    /// Run complete pipeline for all eligible targets and tiers
    RunPipeline {
        #[arg(long)]
        db: String,
        #[arg(long)]
        config: String,
    },

    /// Export R-ready CSV files from SQLite
    ExportR {
        #[arg(long)]
        db: String,
        #[arg(long)]
        out: String,
    },

    /// Export GIS-ready CSV files from SQLite
    ExportGis {
        #[arg(long)]
        db: String,
        #[arg(long)]
        out: String,
    },
}
