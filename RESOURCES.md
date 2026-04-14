## Acquiring JMdict files

Download newest version directly from ftp://ftp.edrdg.org/pub/Nihongo/JMdict_e.gz

## Acquiring KANJIDIC2 files

Download newest version directly from https://www.edrdg.org/kanjidic/kanjidic2.xml.gz

## Acquiring MeCab IPADIC v2.7.0 files

Bound to specific `vibrato` version as described in https://github.com/daac-tools/vibrato/blob/v0.5.2/docs/compile.md
and https://github.com/daac-tools/vibrato/blob/v0.5.2/docs/map.md

Pre-compiled files can be downloaded from https://github.com/daac-tools/vibrato/releases/download/v0.5.0/ipadic-mecab-2_7_0.tar.xz

## Preparing fresh Wiktionary JSON

```bash
# clone repository
git clone https://github.com/tatuylonen/wiktextract.git && \
  git checkout 05c257fdecbc64e73a31a2ca2c0f6cb0ee4c0a68

# apply patch for romaji (https://github.com/tatuylonen/wiktextract/issues/1620)
git apply <<'EOF'
diff --git a/src/wiktextract/extractor/en/example.py b/src/wiktextract/extractor/en/example.py
index a224f3c2..fce55506 100644
--- a/src/wiktextract/extractor/en/example.py
+++ b/src/wiktextract/extractor/en/example.py
@@ -167,7 +167,7 @@ def extract_template_ja_usex(
         )
         example_data["ruby"] = ruby_data
     for span_tag in expanded_node.find_html_recursively(
-        "span", attr_name="class", attr_value="tr"
+        "span", attr_name="class", attr_value="e-transliteration"
     ):
         example_data["roman"] = clean_node(wxr, None, span_tag)
         calculate_bold_offsets(
@@ -177,6 +177,7 @@ def extract_template_ja_usex(
             example_data,
             "bold_roman_offsets",
         )
+        break
     tr_arg = wxr.wtp.parse(
         wxr.wtp.node_to_wikitext(node.template_parameters.get(3, "")),
         expand_all=True,
EOF

# set up venv
python3 -m venv .venv
pip install -U pip && pip install -e .

# download xml dump
wget https://dumps.wikimedia.org/enwiktionary/latest/enwiktionary-latest-pages-articles.xml.bz2

# prepare database
wiktwords --db-path enwiktionary-latest.db --edition en --skip-extraction enwiktionary-latest-pages-articles.xml.bz2

# extract entries
wiktwords --db-path enwiktionary-latest.db --edition en --language-code ja \
  --examples --etymologies --linkages --pronunciations --out enwiktionary-ja.jsonl

wiktwords --db-path enwiktionary-latest.db --edition en --language-code en \
  --examples --etymologies --linkages --pronunciations --out enwiktionary-en.jsonl

# combine both languages, keep only fields used by build-wiktionary, and clean-up
# remaining stale elements such as redirects or unrelated thesaurus entries
cat <<'EOF' > enwiktionary-ja_en.jq
def rels: [.[]? | {word}];
def examples: [.[]? | {text, english, translation, roman}
  | with_entries(select(.value != null))
  | select(.text != null and .text != "")];

select(has("senses") and has("pos") and (.senses | length > 0))
| (.lang_code == "en") as $en
| {word, pos, lang_code, etymology_number, 
   forms: [.forms[]? | {form, ruby, tags} | with_entries(select(.value != null))],
   sounds: [.sounds[]? | select(has("ipa")) | {ipa}],
   synonyms: (.synonyms | rels), antonyms: (.antonyms | rels),
   coordinate_terms: (.coordinate_terms | rels), related: (.related | rels),
   derived: (.derived | rels), hyponyms: (.hyponyms | rels),
   senses: [.senses[] | {glosses, tags,
     synonyms: (.synonyms | rels), antonyms: (.antonyms | rels),
     coordinate_terms: (.coordinate_terms | rels), related: (.related | rels),
     derived: (.derived | rels), hypernyms: (.hypernyms | rels),
     hyponyms: (.hyponyms | rels),
     examples: (if $en then null else .examples | examples end)}
   | with_entries(select(.value != null and .value != []))]}
| with_entries(select(.value != null and .value != []))
EOF
jq -cf enwiktionary-ja_en.jq enwiktionary-ja.jsonl enwiktionary-en.jsonl > enwiktionary-ja_en.jsonl
```
