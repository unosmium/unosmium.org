This is the old design before migration to custom Rust-based generator.

Will probably still be viewable here:
https://5ef93ad106d3b43312c5b13d--unosmium.netlify.app/results/

---

# Unosmium Website

[![Netlify Status](https://api.netlify.com/api/v1/badges/ff03ce31-0235-4742-b95e-389ec3ebb96a/deploy-status)](https://app.netlify.com/sites/unosmium/deploys)

The official website for the Unosmium Scoring System and Unosmium Results.

Visit: [https://unosmium.org/](https://unosmium.org/)

## Unosmium Scoring System

Homepage is a work in progress (aka has not yet been started on). Will be used
to advertise the features of the Unosmium Scoring System and provide
instructions on how to use it.

## Unosmium Results

An [archive](https://unosmium.org/results/) of any tournament results
that have been output as or converted into the
[SciolyFF](https://github.com/unosmium/sciolyff) (Science Olympiad File Format).

### How to view locally

Minimal instructions that will likely need to be modified depending on your
development setup:
```sh
git clone https://github.com/unosmium/unosmium.org.git
bundle install
middleman build
firefox build/results/index.html
```

### Contributing

Contributions of code and tournament results are welcome.

To add new tournament results, make a [Pull
Request](https://help.github.com/en/articles/creating-a-pull-request) that adds
a YAML file in format of [SciolyFF](https://github.com/unosmium/sciolyff) in the
[data directory](/data).

A Google Sheets [input
template](https://docs.google.com/spreadsheets/d/1bkDCZD1NYYsS8L8m_e_cZ2kn9dZm2vufC4U9hNum6Hg)
can be used to generate a CSV file that then can be converted into a SciolyFF
file using the script in [this
repository](https://github.com/unosmium/sciolyff-conversions).  The files
already in the data directory should serve as an example of expected output.
Additionally, the `sciolyff` command line utility should be used to verify the
data files.

After the pull request is merged, the website will automatically generate an
HTML results page that can be viewed by clicking on the appropriate link in the
[site index](https://unosmium.org/results/).
