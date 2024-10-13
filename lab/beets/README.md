# beets

[beets](https://beets.io/) is used to manage my music library.

## Usage

```bash
# Enter virtual environment.
just develop

# Install dependencies & symlink config.
just install

# Confirm that beets is installed.
beet --version
```

## Plugins

* [BadFiles](https://beets.readthedocs.io/en/stable/plugins/badfiles.html) for finding missing and corrupt files.
* [Chroma](https://beets.readthedocs.io/en/stable/plugins/chroma.html) for identifying songs with acoustic fingerprinting.
* [Duplicates](https://beets.readthedocs.io/en/stable/plugins/duplicates.html) for finding duplicate tracks.
* [FetchArt](https://beets.readthedocs.io/en/stable/plugins/fetchart.html) for retrieving album art.
* [MBSync](https://beets.readthedocs.io/en/stable/plugins/mbsync.html) for fetching metadata from MusicBrainz.
* [Scrub](https://beets.readthedocs.io/en/stable/plugins/scrub.html) for cleaning up metadata.
