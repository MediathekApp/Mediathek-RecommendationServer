import urllib.request
import json
from scoring import Scorer
from caching_server import fetch_cached_item_data, push_aggregated_data
from categories import load_programs_with_categories_from_publishers, categorize_items
import config  # <- This is not part of the repository!

# Script configuration
do_aggregate_items=True 
do_aggregate_queries=True
publishers = ["3sat", "arte", "zdf", "ard"]

# 3.1 Load programs
programs_with_categories = load_programs_with_categories_from_publishers(publishers)


# Step 1: Download metrics
HOST = config.RECOMMENDATION_SERVER_HOST
url = "https://{}/counters".format(HOST)
try:
    with urllib.request.urlopen(url) as response:
        data_bytes = response.read()
        view_data = data_bytes.decode('utf-8')
except Exception as e:
    print(f"Failed to download data: {e}")
    exit(1)

# Step 2: Score the strings
scorer = Scorer()
try:
    recommendations = scorer.score(view_data)
except Exception as e:
    print(f"Scoring failed: {e}")
    exit(1)


# Step 3: Aggregate recommendations
if do_aggregate_items:

    max_items = 100 # limit items to process

    items = []
    for rec in recommendations:
        string = rec["string"]
        # Skip all strings that are not URNs
        if not string.startswith("urn:"):
            continue
        item_data = fetch_cached_item_data(string)
        if item_data:
            print(f"Fetched item for {string}: {item_data.get('title', 'No title')}")
            # Optionally add score to the item
            item_data["score"] = rec["score"]
            items.append(item_data)
            if len(items) >= max_items:
                break

    print(f"Fetched {len(items)} items from cache.")

    # 3.2 Filter recommendations based on categories
    items_by_category = categorize_items(items, programs_with_categories)

    # 3.2 Output the items by category
    for category, category_items in items_by_category.items():
        print(f"Category: {category}, Items: {len(category_items)}")

    max_items_per_list = 10  # Limit items per list/category

    # Push aggregated lists to the caching server

    # 3.3 Top items: the highest scored items across all categories
    top_items = items[:max_items_per_list]
    push_aggregated_data({"items": top_items}, urn="urn:mediathek:recommendations:top")

    # 3.4 Most recent items: the most recent items across all categories (last 7 days)
    # The unix epoch timestamp is expected in the "broadcasts" field of each item.
    from datetime import datetime, timedelta
    recent_items = []
    seven_days_ago = datetime.now() - timedelta(days=7)
    for item in items:
        if "broadcasts" in item and item["broadcasts"]:
            broadcast_time = datetime.fromtimestamp(item["broadcasts"])
            if broadcast_time >= seven_days_ago:
                recent_items.append(item)

    recent_items = recent_items[:max_items_per_list]
    # Build a list of categories to attach to the data (take the keys from items_by_category)
    categories = list(items_by_category.keys())
    # Unshift the "recent" category to the front
    categories.insert(0, "recent")
    push_aggregated_data({"items": recent_items, "categories": categories, "currentCategory": "recent"}, urn="urn:mediathek:recommendations:recent")

    # 3.5 Items by category: the highest scored items per category
    for category, items in items_by_category.items():
        top_items_in_category = items[:max_items_per_list]
        push_aggregated_data({"items": top_items_in_category}, urn=f"urn:mediathek:recommendations:{category}")

    # 3.6 Finally, push the list of categories
    push_aggregated_data({"categories": list(items_by_category.keys())}, urn="urn:mediathek:recommendations:categories")


# Step 4: Aggregate queries
if do_aggregate_queries:
    max_queries = 500  # limit queries to process
    # Collect all queries from the recommendations (strings without a urn: or https: prefix)
    queries = [rec["string"] for rec in recommendations if not rec["string"].startswith(("urn:", "https:"))]
    #print(f"Collected {len(queries)} queries from recommendations: {queries[:10]}...")  # Print first 10 queries for debugging
    aggregated_queries = []
    # Go through all queries
    for query in queries:
        # Go through all programs_with_categories and see if their synonyms contain the query
        matching_programs = [program for program in programs_with_categories if query.lower() in (syn.lower() for syn in program.get("synonyms", []))]
        # Warn if there are multiple matches
        if len(matching_programs) > 1:
            print(f"⚠️ Warning: Multiple programs with synonyms found for query '{query}': {[program['urn'] for program in matching_programs]}")
        # If there is a match and it has a name, use that as the query
        query = next((program["name"] for program in matching_programs if "name" in program), query)
        # Add the query to the aggregated queries list
        aggregated_queries.append({
            "query": query,
            "programs": [program["urn"] for program in matching_programs]
        })
        # If we reach the max queries, stop
        if len(aggregated_queries) >= max_queries:
            print(f"Reached max queries limit of {max_queries}. Stopping aggregation.")
            break
    
    # Push the aggregated queries to the caching server
    push_aggregated_data({"queries": aggregated_queries}, urn="urn:mediathek:recommendations:queries")


print("Finished pushing aggregated data to the caching server.")