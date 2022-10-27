window.BENCHMARK_DATA = {
  "lastUpdate": 1666880602583,
  "repoUrl": "https://github.com/eclipse/chariott",
  "entries": {
    "ESS Benchmark": [
      {
        "commit": {
          "author": {
            "email": "code@raboof.com",
            "name": "Atif Aziz",
            "username": "atifaziz"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "02ae33d40eb06dbff4a56f7ee150cca84bd62cec",
          "message": "Trim trailing whitespace from license/notice (#6)",
          "timestamp": "2022-10-27T16:04:50+02:00",
          "tree_id": "6debd077818e3dc7447937b96cf58a9205e35e31",
          "url": "https://github.com/eclipse/chariott/commit/02ae33d40eb06dbff4a56f7ee150cca84bd62cec"
        },
        "date": 1666880600437,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 416668,
            "range": "± 54216",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2776218,
            "range": "± 267911",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25725757,
            "range": "± 1648861",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3520546,
            "range": "± 236843",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26010394,
            "range": "± 2063344",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 248809404,
            "range": "± 12330086",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}