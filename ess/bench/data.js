window.BENCHMARK_DATA = {
  "lastUpdate": 1667580112857,
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
          "id": "c32019649bd315dbc28a400e5679e4f56bb5fcf5",
          "message": "chore: Bump serde_json from 1.0.86 to 1.0.87 (#2)\n\nBumps [serde_json](https://github.com/serde-rs/json) from 1.0.86 to 1.0.87.\r\n- [Release notes](https://github.com/serde-rs/json/releases)\r\n- [Commits](https://github.com/serde-rs/json/compare/v1.0.86...v1.0.87)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: serde_json\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2022-11-02T08:33:01+01:00",
          "tree_id": "37b666f4a1ac97bd929c657cd33be344280ab8b7",
          "url": "https://github.com/eclipse/chariott/commit/c32019649bd315dbc28a400e5679e4f56bb5fcf5"
        },
        "date": 1667374721473,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 508782,
            "range": "± 73757",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3238827,
            "range": "± 267191",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29510212,
            "range": "± 2109445",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4727512,
            "range": "± 367415",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 30414902,
            "range": "± 2087973",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 295442569,
            "range": "± 19339481",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "daniellueddecke@users.noreply.github.com",
            "name": "Daniel Lueddecke",
            "username": "daniellueddecke"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6044c0cf92768a2dcc2a4f618744dbc98dc2305c",
          "message": "Fix Chariott typos in read-me doc (#17)",
          "timestamp": "2022-11-02T16:08:08+01:00",
          "tree_id": "a20d8fb6f9a80575eb475007d2a83e4b6e8a0ae2",
          "url": "https://github.com/eclipse/chariott/commit/6044c0cf92768a2dcc2a4f618744dbc98dc2305c"
        },
        "date": 1667411085634,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 448444,
            "range": "± 16896",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2657547,
            "range": "± 13989",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24931263,
            "range": "± 39014",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4151764,
            "range": "± 63867",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26517474,
            "range": "± 99115",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 252429001,
            "range": "± 3298796",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "22341213+bastbu@users.noreply.github.com",
            "name": "Bastian Burger",
            "username": "bastbu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "13e5a5ea488d1df51795476bd3e885f77922cffc",
          "message": "Cancel in-progress CI workflow runs for open PRs (#19)",
          "timestamp": "2022-11-03T11:01:42+01:00",
          "tree_id": "4ebaa9727ad9319536ec83cc8a543ebd61da9881",
          "url": "https://github.com/eclipse/chariott/commit/13e5a5ea488d1df51795476bd3e885f77922cffc"
        },
        "date": 1667473534433,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 345976,
            "range": "± 10138",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 1970902,
            "range": "± 4508",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 18628257,
            "range": "± 60033",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3067539,
            "range": "± 10777",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 19397823,
            "range": "± 24650",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 185456368,
            "range": "± 1656197",
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
          "id": "f26a3b1c2ce193d4a218cd64449962840cdbd2a7",
          "message": "Fix read-me section link + title typo (#21)",
          "timestamp": "2022-11-03T11:45:48+01:00",
          "tree_id": "aeb4fa989914f39e998f067f7b8f8bb7ce499eeb",
          "url": "https://github.com/eclipse/chariott/commit/f26a3b1c2ce193d4a218cd64449962840cdbd2a7"
        },
        "date": 1667475966671,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 394848,
            "range": "± 7574",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2705772,
            "range": "± 4002",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25999481,
            "range": "± 263117",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3612917,
            "range": "± 19133",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26654807,
            "range": "± 68506",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 258633154,
            "range": "± 275444",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "22341213+bastbu@users.noreply.github.com",
            "name": "Bastian Burger",
            "username": "bastbu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "73b3ab27bf96b603aad98b65e369e64dd18fbda2",
          "message": "Fix CI for PRs from clones (#20)",
          "timestamp": "2022-11-03T13:10:01+01:00",
          "tree_id": "10eb63b9fd8043260f77d5f19cc1a20d647427d0",
          "url": "https://github.com/eclipse/chariott/commit/73b3ab27bf96b603aad98b65e369e64dd18fbda2"
        },
        "date": 1667479107729,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 453182,
            "range": "± 8570",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2648265,
            "range": "± 9637",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24963454,
            "range": "± 48027",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4173479,
            "range": "± 21213",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26289707,
            "range": "± 327666",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 251763480,
            "range": "± 839591",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "22341213+bastbu@users.noreply.github.com",
            "name": "Bastian Burger",
            "username": "bastbu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "387f4c7acc91b2ffe37f03ca933b6f3d94815f14",
          "message": "Add unit tests cases for capitalized system upsert (#14)",
          "timestamp": "2022-11-03T18:26:38+01:00",
          "tree_id": "2e8cb72509a5cd176999662a83778babb8060eb3",
          "url": "https://github.com/eclipse/chariott/commit/387f4c7acc91b2ffe37f03ca933b6f3d94815f14"
        },
        "date": 1667496730853,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 490391,
            "range": "± 59462",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2980683,
            "range": "± 263317",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 27779823,
            "range": "± 1687648",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4179727,
            "range": "± 327719",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 29873159,
            "range": "± 2175910",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 268681889,
            "range": "± 13692145",
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
          "id": "8a7a660b503d91439483b5cd74abca64cf4201de",
          "message": "Add registration watchdog (#13)",
          "timestamp": "2022-11-03T18:41:33+01:00",
          "tree_id": "38918606540cfd9676f3b1cc681960fb96fc4b7d",
          "url": "https://github.com/eclipse/chariott/commit/8a7a660b503d91439483b5cd74abca64cf4201de"
        },
        "date": 1667497590711,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 447012,
            "range": "± 15460",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2645085,
            "range": "± 7228",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24930696,
            "range": "± 293705",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3911377,
            "range": "± 70404",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26152879,
            "range": "± 104465",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 251333090,
            "range": "± 755416",
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
          "id": "23ff8351d6926ca4a0baa47f569cc05cdd9ab745",
          "message": "chore: Bump base64 from 0.13.0 to 0.13.1 (#4)\n\nBumps [base64](https://github.com/marshallpierce/rust-base64) from 0.13.0 to 0.13.1.\r\n- [Release notes](https://github.com/marshallpierce/rust-base64/releases)\r\n- [Changelog](https://github.com/marshallpierce/rust-base64/blob/master/RELEASE-NOTES.md)\r\n- [Commits](https://github.com/marshallpierce/rust-base64/compare/v0.13.0...v0.13.1)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: base64\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2022-11-04T10:15:06+01:00",
          "tree_id": "1ffb8e8a26dfd4fd68f8062c10b02f9f189a7653",
          "url": "https://github.com/eclipse/chariott/commit/23ff8351d6926ca4a0baa47f569cc05cdd9ab745"
        },
        "date": 1667553624054,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 475450,
            "range": "± 9702",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2660515,
            "range": "± 24462",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24927242,
            "range": "± 202674",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3970978,
            "range": "± 63270",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26310689,
            "range": "± 111051",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 251996554,
            "range": "± 458908",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "22341213+bastbu@users.noreply.github.com",
            "name": "Bastian Burger",
            "username": "bastbu"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8af649426d8c14205650118a5d26ba9f51e7c5aa",
          "message": "Support subscription to registry changes (#15)",
          "timestamp": "2022-11-04T17:37:04+01:00",
          "tree_id": "a330a95278865ac4b683de6c480485b6d61e95a2",
          "url": "https://github.com/eclipse/chariott/commit/8af649426d8c14205650118a5d26ba9f51e7c5aa"
        },
        "date": 1667580112276,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 413820,
            "range": "± 5390",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2713860,
            "range": "± 6367",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25887296,
            "range": "± 525345",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3867441,
            "range": "± 80679",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26666889,
            "range": "± 65390",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 258088681,
            "range": "± 400245",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}