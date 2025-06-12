import json
import hashlib
import urllib.parse
import urllib.request
import sys
import config  # <- This is not part of the repository!

CACHING_SERVER_URL = config.CACHING_SERVER_URL

def process_aliases(publisher_id, json_file_path='data.json'):
    try:
        # Step 1: Load JSON
        with open(json_file_path, 'r', encoding='utf-8') as f:
            entries = json.load(f)

        # Step 2–3: Process each entry
        for entry in entries:
            queries = [entry['name']] + entry.get('synonyms', [])
            queries = [q.lower() for q in queries]

            for query in queries:
                # Create percent-encoded query
                encoded_query = urllib.parse.quote(query, safe='')
                query_urn = f"urn:mediathek:search:lookup:{encoded_query}"

                # SHA256 hashes
                query_hash = hashlib.sha256(query_urn.encode('utf-8')).hexdigest()
                entry_hash = hashlib.sha256(entry['urn'].encode('utf-8')).hexdigest()

                # Compose URL and payload
                url = f"{CACHING_SERVER_URL}/aliases/{query_hash}.jsondfl"
                data = f"{entry_hash}.jsondfl".encode('utf-8')

                # For testing purposes, output the URL and data
                # and stop the script:
                print(f"Processing alias for query: '{query}'")
                # print(f"Encoded Query: '{encoded_query}'")
                # print(f"Query URN: {query_urn}")
                # print(f"Query Hash: {query_hash}")
                # print(f"Entry URN: {entry['urn']}")
                # print(f"Entry Hash: {entry_hash}")
                # print(f"URL: {url}")
                # print(f"Data: {data}")
                # print("-" * 40)
                # return

                # Build and send the PUT request
                req = urllib.request.Request(url, data=data, method='PUT')
                req.add_header('Content-Type', 'text/plain; charset=utf-8')

                try:
                    with urllib.request.urlopen(req) as response:
                        status = response.getcode()
                        if status == 200:
                            print(f"✅ Alias added for: '{query}' -> '{entry['urn']}'")
                        else:
                            print(f"⚠️ Unexpected response for '{query}': {status}")
                except urllib.error.HTTPError as e:
                    print(f"❌ HTTPError for '{query}': {e.code} - {e.reason}")
                except urllib.error.URLError as e:
                    print(f"❌ URLError for '{query}': {e.reason}")

            # return

    except Exception as e:
        print(f"❌ Error: {e}")

# CLI usage
if __name__ == '__main__':
    if len(sys.argv) < 3:
        print("Usage: python script.py <publisherID> <data.json>")
        sys.exit(1)

    publisher_id = sys.argv[1]
    json_file_path = sys.argv[2]
    process_aliases(publisher_id, json_file_path=json_file_path)
