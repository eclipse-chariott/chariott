window.BENCHMARK_DATA = {
  "lastUpdate": 1667313435802,
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
      },
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
          "id": "521ed8233ca816942f3c40b19001a5e0f579cb10",
          "message": "Trim trailing space in notices file during generation (#7)",
          "timestamp": "2022-10-28T16:03:06+02:00",
          "tree_id": "462dde962f32d26623081fdf3a7455260317039a",
          "url": "https://github.com/eclipse/chariott/commit/521ed8233ca816942f3c40b19001a5e0f579cb10"
        },
        "date": 1666967246585,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 501488,
            "range": "± 49864",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3143614,
            "range": "± 110734",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 30548073,
            "range": "± 909177",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4087736,
            "range": "± 207765",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 30478929,
            "range": "± 1142510",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 298166544,
            "range": "± 5692370",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "ea87ae3555251b5b8e12459266b6ed9ae4ff9163",
          "message": "Remove invalid revision from Git blame ignore file (#10)",
          "timestamp": "2022-10-31T10:32:39+01:00",
          "tree_id": "5b74292c7ecacb4ddfed04473437dda2aaf2529e",
          "url": "https://github.com/eclipse/chariott/commit/ea87ae3555251b5b8e12459266b6ed9ae4ff9163"
        },
        "date": 1667209092439,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 422029,
            "range": "± 8839",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2714266,
            "range": "± 7730",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 26089359,
            "range": "± 26885",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3865490,
            "range": "± 120673",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 27418231,
            "range": "± 69419",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 260155162,
            "range": "± 2789548",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "113ee4457fba295351da647b8708ec9bcf5a2af4",
          "message": "Fix notice generation script to quote PR title (#11)",
          "timestamp": "2022-10-31T14:47:46+01:00",
          "tree_id": "4df68469588a8e4bf8003982fea7950c35783a60",
          "url": "https://github.com/eclipse/chariott/commit/113ee4457fba295351da647b8708ec9bcf5a2af4"
        },
        "date": 1667224352268,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 432990,
            "range": "± 16298",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2641612,
            "range": "± 12161",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24930303,
            "range": "± 389933",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3847469,
            "range": "± 60575",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26018523,
            "range": "± 131825",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 248502442,
            "range": "± 2088378",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "patrick.schuler@microsoft.com",
            "name": "Patrick Schuler",
            "username": "p-schuler"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8cbe907a3297d0b944d57d5878ae8fef691bed41",
          "message": "chore: remove PR title linting (#12)\n\nThis was used to try to force the use of conventional commit messages. It only enforces the PR title, but not the commit message itself. The flow is not clear and to avoid confusion, we want to disable this.",
          "timestamp": "2022-11-01T11:25:54+01:00",
          "tree_id": "01f781b36c45e586f143e71ae7f8bc7d44fba3dd",
          "url": "https://github.com/eclipse/chariott/commit/8cbe907a3297d0b944d57d5878ae8fef691bed41"
        },
        "date": 1667298733757,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 548446,
            "range": "± 5351",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3138737,
            "range": "± 34442",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29812076,
            "range": "± 287436",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4591087,
            "range": "± 143546",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 31635297,
            "range": "± 475779",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 298665691,
            "range": "± 1178573",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "46e700923cfadbddf4aaf6c8c14e2c5c5dd0e9d6",
          "message": "chore: Bump serde from 1.0.145 to 1.0.147 (#1)\n\nBumps [serde](https://github.com/serde-rs/serde) from 1.0.145 to 1.0.147.\r\n- [Release notes](https://github.com/serde-rs/serde/releases)\r\n- [Commits](https://github.com/serde-rs/serde/compare/v1.0.145...v1.0.147)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: serde\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2022-11-01T14:11:54+01:00",
          "tree_id": "bda12951023a5d5845102036906b508d269df880",
          "url": "https://github.com/eclipse/chariott/commit/46e700923cfadbddf4aaf6c8c14e2c5c5dd0e9d6"
        },
        "date": 1667310754450,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 481671,
            "range": "± 12598",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2618890,
            "range": "± 14771",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24969042,
            "range": "± 404178",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4117371,
            "range": "± 34824",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26308169,
            "range": "± 317385",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 251546380,
            "range": "± 571401",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6b4c4ca79f5b4bf3c67fba55dd0170926f920b35",
          "message": "chore: Bump async-trait from 0.1.57 to 0.1.58 (#3)\n\nBumps [async-trait](https://github.com/dtolnay/async-trait) from 0.1.57 to 0.1.58.\r\n- [Release notes](https://github.com/dtolnay/async-trait/releases)\r\n- [Commits](https://github.com/dtolnay/async-trait/compare/0.1.57...0.1.58)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: async-trait\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2022-11-01T14:37:42+01:00",
          "tree_id": "c51562cd637d1405177a970aefc3650d85cb37b6",
          "url": "https://github.com/eclipse/chariott/commit/6b4c4ca79f5b4bf3c67fba55dd0170926f920b35"
        },
        "date": 1667313024399,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 531359,
            "range": "± 45146",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3151496,
            "range": "± 70354",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29805566,
            "range": "± 270351",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4512527,
            "range": "± 77386",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 31618251,
            "range": "± 264734",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 300192254,
            "range": "± 916313",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b0a9f1481f5fa63fd8d53c2095782ee09465cb96",
          "message": "chore: Bump futures-util from 0.3.24 to 0.3.25 (#5)\n\nBumps [futures-util](https://github.com/rust-lang/futures-rs) from 0.3.24 to 0.3.25.\r\n- [Release notes](https://github.com/rust-lang/futures-rs/releases)\r\n- [Changelog](https://github.com/rust-lang/futures-rs/blob/master/CHANGELOG.md)\r\n- [Commits](https://github.com/rust-lang/futures-rs/compare/0.3.24...0.3.25)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: futures-util\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2022-11-01T14:48:04+01:00",
          "tree_id": "f43d9958620bde6f10d8d0762e51d3820decab2b",
          "url": "https://github.com/eclipse/chariott/commit/b0a9f1481f5fa63fd8d53c2095782ee09465cb96"
        },
        "date": 1667313433889,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 516296,
            "range": "± 16995",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3155663,
            "range": "± 77486",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29902834,
            "range": "± 117284",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4425527,
            "range": "± 44113",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 31711997,
            "range": "± 305691",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 299554251,
            "range": "± 2201656",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}