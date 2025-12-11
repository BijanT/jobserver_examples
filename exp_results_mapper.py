#!/usr/bin/env python3
import sys
import json

json_input = None
for line in sys.stdin:
	json_data = json.loads(line)

# The ID for this particular job
jid = json_data['jid']
# Where the results files for this job are located
results_prefix = json_data['results_path']
params_filename = results_prefix + "params"
runtime_filename = results_prefix + "time"

# Read the parameters json file
params_file = open(params_filename)
params_data = json.load(params_file)

iteration_time = str(params_data['time'])
iterations = str(params_data['iterations'])

# Read the runtime file
runtime_file = open(runtime_filename)
runtime = runtime_file.readline().strip("\n")

# Construct the output and output is as a JSON string
outdata = {
	"Iteration Time (s)": iteration_time,
	"Iterations": iterations,
	"Runtime (s)": runtime,
	# I like to include the Job ID and results location in the output
	# in case I want to look at the job more closely.
	"Job ID": jid,
	"Results Location": results_prefix,
}
print(json.dumps(outdata))