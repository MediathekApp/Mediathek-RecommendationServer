import urllib.request
import json
import hashlib
import zlib
import config  # <- This is not part of the repository!

CACHING_SERVER_URL = config.CACHING_SERVER_URL

def fetch_cached_item_data(urn):
    # Step 1: Hash URN to SHA256
    sha256_hash = hashlib.sha256(urn.encode('utf-8')).hexdigest()
    url = f"{CACHING_SERVER_URL}/keys/{sha256_hash}.jsondfl"

    try:
        with urllib.request.urlopen(url) as response:
            compressed = response.read()
    except Exception as e:
        print(f"Failed to fetch cache for {urn}: {e} (URL: {url})")
        return None

    # Step 2: Decompress using raw deflate (no headers)
    try:
        decompressed_bytes = zlib.decompress(compressed, -zlib.MAX_WBITS)
        item_json = decompressed_bytes.decode('utf-8')
        return json.loads(item_json)
    except Exception as e:
        print(f"Failed to decompress or decode item for {urn}: {e}")
        return None

def push_aggregated_data(data, urn="urn:mediathek:recommendations"):
    # Step 1: Convert JSON to bytes
    json_bytes = json.dumps(data, separators=(",", ":")).encode('utf-8')  # Minified
    
    # Step 2: Compress using raw deflate (no zlib headers)
    compressed = zlib.compress(json_bytes)
    compressed_raw = compressed[2:-4]  # Strip zlib header and checksum

    # OR use this safer one-liner instead:
    compressed_raw = zlib.compress(json_bytes, level=9)[2:-4]

    # Step 3: Create the SHA256 hash of the URN
    sha256_hash = hashlib.sha256(urn.encode('utf-8')).hexdigest()
    url = f"{CACHING_SERVER_URL}/keys/{sha256_hash}.jsondfl"

    # Step 4: PUT request
    try:
        req = urllib.request.Request(url, data=compressed_raw, method="PUT")
        req.add_header("Content-Type", "application/octet-stream")
        with urllib.request.urlopen(req) as response:
            status = response.status
            print(f"✅ PUT response: {status} for {urn} (URL: {url})")
            return status == 200
    except Exception as e:
        print(f"Failed to PUT recommendations: {e}")
        return False
