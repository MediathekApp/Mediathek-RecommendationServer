import json
import os

def load_program_categories(filename):
    # Get the relative path to the file
    relative_path = os.path.join(os.path.dirname(__file__), filename)
    try:
        with open(relative_path, 'r') as file:
            return json.load(file)
    except FileNotFoundError:
        print(f"Warning: {filename} not found, skipping category loading.")
        return []

def load_program_categories_for_publisher(publisher):
    filename = f"programs/{publisher}.json"
    return load_program_categories(filename)

def load_programs_with_categories_from_publishers(publishers):
    categories = []
    for publisher in publishers:
        category_data = load_program_categories_for_publisher(publisher)
        if category_data:
            categories.extend(category_data)
    return categories

def categorize_items(items, program_categories):
    items_by_category = {}
    for item in items:
        # get the program ID from the item (program?.id)
        program_id = item.get("program", {}).get("id")
        if not program_id:
            print(f"⚠️ Warning: Item {item.get('title', 'No title')} has no program ID, skipping.")
            continue
        # publisher_id is the lowercased item["publisher"], for example "arte" for "ARTE"
        publisher_id = item.get("publisher", "").lower()
        if not publisher_id:
            print(f"⚠️ Warning: Item {item.get('title', 'No title')} has no publisher ID, skipping.")
            continue
        urn = "urn:mediathek:" + publisher_id + ":program:" + program_id
        program = next((prog for prog in program_categories if prog["urn"] == urn), None)
        if program:
            categories = program.get("categories", [])
            print(f"Item {item.get('title', 'No title')} categorized under {categories} for program {program_id}.")
            for category in categories:
                if category not in items_by_category:
                    items_by_category[category] = []
                items_by_category[category].append(item)
        else:
            print(f"⚠️ Warning: No program found for URN {urn}, skipping item {item.get('title', 'No title')}.")
    return items_by_category
