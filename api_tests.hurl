# API Tests for mag_server
# Run with: hurl --test api_tests.hurl
# Assumes server is running on http://localhost:3000/api

# Test 1: Create a new project
POST http://localhost:3000/api/projects/create
Content-Type: application/json
{
    "project_id": "test_project_1",
    "methylation_data_path": "/home/sebdal/workdir/contam-map/test_data/expected_out_median.tsv",
    "contig_bin_path": "/home/sebdal/workdir/contam-map/test_data/contig_bin.tsv",
    "output_path": "/tmp/test_output_1"
}

HTTP 200


# Test 2: Try to create a duplicate project (should fail)
POST http://localhost:3000/api/projects/create
Content-Type: application/json
{
    "project_id": "test_project_1",
    "methylation_data_path": "/home/sebdal/workdir/contam-map/test_data/expected_out_median.tsv",
    "contig_bin_path": "/home/sebdal/workdir/contam-map/test_data/contig_bin.tsv",
    "output_path": "/tmp/test_output_1"
}

HTTP 409


# Test 3: Get bins for an existing project
GET http://localhost:3000/api/projects/bins?project_id=test_project_1

HTTP 200
[Asserts]
header "Content-Type" contains "application/json"
jsonpath "$" isCollection


# Test 4: Get bins for a non-existent project (should fail)
GET http://localhost:3000/api/projects/bins?project_id=nonexistent_project

HTTP 404


# Test 5: Create another project with different ID
POST http://localhost:3000/api/projects/create
Content-Type: application/json
{
    "project_id": "test_project_2",
    "methylation_data_path": "/home/sebdal/workdir/contam-map/test_data/expected_out_median.tsv",
    "contig_bin_path": "/home/sebdal/workdir/contam-map/test_data/contig_bin.tsv",
    "output_path": "/tmp/test_output_2"
}

HTTP 200


# Test 6: Get bins for the second project
GET http://localhost:3000/api/projects/bins?project_id=test_project_2

HTTP 200
[Asserts]
jsonpath "$" isCollection


# Test 7: Missing required field in create request (should fail)
POST http://localhost:3000/api/projects/create
Content-Type: application/json
{
    "project_id": "incomplete_project",
    "methylation_data_path": "/path/to/methylation/data"
}

HTTP 422


# Test 8: Missing query parameter (should fail)
GET http://localhost:3000/api/projects/bins

HTTP 400
