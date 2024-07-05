import os

methods = ["autohog", "area", "mv_aware"]

for method in methods:
	benchmarks = []
	folder = f'./results_lbf/{method}/'
	for file in os.listdir( folder ):
		if file.endswith( '.lbf' ):
			benchmarks.append( os.path.join( folder, file ) )
	benchmarks.sort()

	for benchmark in benchmarks:
		command = f'python3 estimator.py {benchmark}'
		print( command )
		os.system( command )

	print('-' * 40)