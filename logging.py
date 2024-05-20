import json
import sys


def beautify_log(log):
    try:
        # Split the log into the non-JSON part and the JSON part
        non_json_part, json_part = log.split(' ', 2)[0:2], log.split(' ', 2)[2]
        non_json_part_str = ' '.join(non_json_part)
        
        # Parse and pretty-print the JSON part
        log_dict = json.loads(json_part)
        pretty_json = json.dumps(log_dict, indent=4)
        
        # Combine the non-JSON part and the pretty-printed JSON part
        formatted_log = f"{non_json_part_str} {pretty_json}"
        return formatted_log
    except json.JSONDecodeError:
        # If the log is not in the expected format, return it as is
        return log

for line in sys.stdin:
    print(beautify_log(line.strip()))
