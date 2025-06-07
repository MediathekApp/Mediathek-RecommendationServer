import json
from collections import defaultdict

class Scorer:
    def __init__(self, bucket_weights=None):
        self.bucket_weights = bucket_weights or {
            "this_hour": 1.0,
            "last_hour": 0.75,
            "hour_minus_2": 0.5,
            "today": 0.25,
            "yesterday": 0.1,
            "day_minus_2": 0.05
        }

    def score(self, json_input):
        """Accepts a JSON string or dict and returns a sorted list of scores."""
        if isinstance(json_input, str):
            try:
                data = json.loads(json_input)
            except json.JSONDecodeError:
                raise ValueError("Invalid JSON input")
        elif isinstance(json_input, dict):
            data = json_input
        else:
            raise TypeError("Input must be a JSON string or dictionary")

        scores = defaultdict(float)

        for bucket, strings in data.items():
            weight = self.bucket_weights.get(bucket, 0)
            for string, count in strings.items():
                scores[string] += count * weight

        sorted_scores = sorted(scores.items(), key=lambda x: x[1], reverse=True)

        return [
            {"string": string, "score": round(score, 2)}
            for string, score in sorted_scores
        ]