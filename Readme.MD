# rpg_map_to_html

Generates html documents from rpg maps. The resulting html files can be printed to a pdf file with
most major browsers.

# Special Thanks

Thanks to Gavin Norman and his brilliant Dolmenwood! Your old players in Germany say hello :)

# Examples

See the `examples` folder to see what the input and resulting PDF can look like.

# Who is this for?

For GMs and players who want to share location-based information with each other. This helps with
any exploration based game or hexcrawl, but is especially useful for open tables. Instead of
reading through multiple session logs, new or returning players can just browse through the locations
the party has discovered.

The resulting file is styled loosely like the Dolmenwood Campaign Book. A table-of-contents map
at the beginning of the file links to all discovered cells, and each cell's page contains a hyperlinked
map cutout showing its surroundings.

# Usage

## Make a map

First, you need a map! The app was built for hex maps from Hexographer, but any hex map will work.

I would recommend not revealing the full map to the players, rather only uncovering hexes bit-by-bit
as the players explore. Per default, the app will not generate any documentation for 'empty' (white) hexes.
You can reconfigure what color counts as 'empty'.

## Convert your map to a new html file

Now run the app. It will start in interactive mode and will ask you for the input needed to
transform the map into an html file. The resulting files will be created in a target directory
of your choice. Of particular importance are

* rpg_map_doc.html - This is the html file that you can then print to a PDF.
* map_content.md - This is a markdown file that you and your players can edit to document the map.

## Document your map

You or your players can edit the `map_content.md` file to write documentation for the map cells
that they have discovered. A page is generated for every non-empty cell that does not yet have a cell
page. Generated pages contain instructions on how `map_content.md` is used.

Once you've added your own content, simply re-run the app and it will update the html for you.

## Print to PDF

All major browsers can then print the html to a pdf file.

## Share with your party

You can share the your map documentation PDF and the `map_content.md` with your players. After each
section, you can update the map image (if the players have explored further) and the `map_content` and rerun the app,
generating an updated html and PDF.

## Advanced Users

Instead of using the default interactive mode, you can supply a [Config] JSON directly to the app.
This lets you change the details of how everything is generated, up to even using your own
html templates.

## Supported Geometries

Currently only Hex maps and arbitrary manual maps are supported. If there's interest, I'll add
support for square maps, triangles, etc.