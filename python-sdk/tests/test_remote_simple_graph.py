import unittest
from typing import List, Mapping

from pydantic import BaseModel

from indexify import Graph, create_client
from indexify.functions_sdk.data_objects import File
from indexify.functions_sdk.indexify_functions import indexify_function


@indexify_function()
def extractor_a(url: str) -> File:
    """
    Random description of extractor_a
    """
    print(f"extractor_a called with url: {url}")
    assert url == "https://example.com"
    assert isinstance(url, str)
    return File(data=bytes(b"hello" * 100), mime_type="text/plain")


class FileChunk(BaseModel):
    data: bytes
    start: int
    end: int


@indexify_function()
def extractor_b(file: File) -> List[FileChunk]:
    return [
        FileChunk(data=file.data, start=0, end=5),
        FileChunk(data=file.data, start=5, end=len(file.data)),
    ]


class SomeMetadata(BaseModel):
    metadata: Mapping[str, str]


@indexify_function()
def extractor_c(file_chunk: FileChunk) -> SomeMetadata:
    return SomeMetadata(metadata={"a": "b", "c": "d"})


def create_graph_a():
    graph = Graph(
        name="graph_a", description="description of graph_a", start_node=extractor_a
    )
    graph = graph.add_edge(extractor_a, extractor_b)
    graph = graph.add_edge(extractor_b, extractor_c)
    return graph


class TestRemoteClient(unittest.TestCase):
    def test_register_graph(self):
        # Register graph
        client = create_client()
        graph = create_graph_a()
        client.register_compute_graph(graph)

        # Get graph Defintion
        compute_graph = client.graph("graph_a")
        self.assertEqual(compute_graph.name, "graph_a")

        # Invoke Graph
        invocation_id = client.invoke_graph_with_object(
            graph.name,
            block_until_done=True,
            url="https://example.com",
        )
        print("getting outputs")
        outputs = client.graph_outputs(
            graph.name, invocation_id, fn_name=extractor_c.name
        )
        print(f"Outputs: {outputs}")


if __name__ == "__main__":
    unittest.main()
