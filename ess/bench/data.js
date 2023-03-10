window.BENCHMARK_DATA = {
  "lastUpdate": 1678471194782,
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
      },
      {
        "commit": {
          "author": {
            "email": "41898282+github-actions[bot]@users.noreply.github.com",
            "name": "github-actions[bot]",
            "username": "github-actions[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fa317e1d2acdbd17966f7116e33c862f7a537e05",
          "message": "chore: Notice file change (#23)",
          "timestamp": "2022-11-08T09:11:31+01:00",
          "tree_id": "93cb7b264f98f1afc8561f808e0ef7219eedb8a8",
          "url": "https://github.com/eclipse/chariott/commit/fa317e1d2acdbd17966f7116e33c862f7a537e05"
        },
        "date": 1667895446256,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 536077,
            "range": "± 24692",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3199803,
            "range": "± 106666",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 30337813,
            "range": "± 64260",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4418054,
            "range": "± 83249",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 31809762,
            "range": "± 551063",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 299826240,
            "range": "± 733853",
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
          "id": "2da7c91a9b98e091ae55103ba0f146de449e58bb",
          "message": "chore: Bump prost from 0.11.0 to 0.11.2 (#28)\n\nBumps [prost](https://github.com/tokio-rs/prost) from 0.11.0 to 0.11.2.\r\n- [Release notes](https://github.com/tokio-rs/prost/releases)\r\n- [Commits](https://github.com/tokio-rs/prost/compare/v0.11.0...v0.11.2)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: prost\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2022-11-09T10:11:54+01:00",
          "tree_id": "203f77382a262a2b2908a8f12b1ac444efd8daeb",
          "url": "https://github.com/eclipse/chariott/commit/2da7c91a9b98e091ae55103ba0f146de449e58bb"
        },
        "date": 1667987029006,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 440113,
            "range": "± 10024",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2646170,
            "range": "± 15079",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24909400,
            "range": "± 248969",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3806202,
            "range": "± 73786",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26219884,
            "range": "± 197517",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 249467075,
            "range": "± 634973",
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
          "id": "2ec16c5b4c31eeffe0689281b1eb0490b148cbc3",
          "message": "chore: Bump anyhow from 1.0.65 to 1.0.66 (#25)\n\nBumps [anyhow](https://github.com/dtolnay/anyhow) from 1.0.65 to 1.0.66.\r\n- [Release notes](https://github.com/dtolnay/anyhow/releases)\r\n- [Commits](https://github.com/dtolnay/anyhow/compare/1.0.65...1.0.66)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: anyhow\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2022-11-09T10:13:59+01:00",
          "tree_id": "47b3dc2b8bc0bd967961698d1e7d6a2404ceed2a",
          "url": "https://github.com/eclipse/chariott/commit/2ec16c5b4c31eeffe0689281b1eb0490b148cbc3"
        },
        "date": 1667987493370,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 528777,
            "range": "± 37818",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3087215,
            "range": "± 112253",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 28435767,
            "range": "± 1076461",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4799275,
            "range": "± 249625",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 30553622,
            "range": "± 741579",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 300321071,
            "range": "± 2205074",
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
          "id": "07ee91d9db1499ce61495b3cedd96386dfcff802",
          "message": "Remove redunant casts in value conversions (#30)",
          "timestamp": "2022-11-09T10:58:53+01:00",
          "tree_id": "0481ad32b2ccae257bb2ce4e6c35441e6710cc14",
          "url": "https://github.com/eclipse/chariott/commit/07ee91d9db1499ce61495b3cedd96386dfcff802"
        },
        "date": 1667988464590,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 437977,
            "range": "± 5816",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2651537,
            "range": "± 12061",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24902377,
            "range": "± 594316",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3849701,
            "range": "± 34718",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 25920142,
            "range": "± 107721",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 251482827,
            "range": "± 1433306",
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
          "id": "8a69f1b318dd50f0605c2c0b158e0f8c9c2cd99b",
          "message": "Add value conversions for floating-point primitives (#31)",
          "timestamp": "2022-11-09T10:59:12+01:00",
          "tree_id": "8705df3fea10f98e56a382db8987e5a9187287b3",
          "url": "https://github.com/eclipse/chariott/commit/8a69f1b318dd50f0605c2c0b158e0f8c9c2cd99b"
        },
        "date": 1667988916442,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 456392,
            "range": "± 11693",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2608029,
            "range": "± 17805",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 24827949,
            "range": "± 297524",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4295154,
            "range": "± 28252",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26246418,
            "range": "± 130106",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 247261838,
            "range": "± 578422",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "41898282+github-actions[bot]@users.noreply.github.com",
            "name": "github-actions[bot]",
            "username": "github-actions[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f2e3f37890dc62da98150daff98112a73b8f45e7",
          "message": "New notice file (#36)",
          "timestamp": "2022-11-14T14:10:38+01:00",
          "tree_id": "f59313dbb7dfb3d1493eb620966f90efa2306f99",
          "url": "https://github.com/eclipse/chariott/commit/f2e3f37890dc62da98150daff98112a73b8f45e7"
        },
        "date": 1668431802323,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 453232,
            "range": "± 17063",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3036582,
            "range": "± 142120",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29017054,
            "range": "± 1228963",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4549450,
            "range": "± 369160",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 29536729,
            "range": "± 1477442",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 297555312,
            "range": "± 14078506",
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
          "id": "e05a4f60562752c88af17073a7dd41b39170a920",
          "message": "Upgrade to Rust 1.65 (#37)",
          "timestamp": "2022-11-14T15:53:37+01:00",
          "tree_id": "39009ae1e7b512b300d8ca680d6f66d4f6c2519e",
          "url": "https://github.com/eclipse/chariott/commit/e05a4f60562752c88af17073a7dd41b39170a920"
        },
        "date": 1668438266569,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 428890,
            "range": "± 15835",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2439680,
            "range": "± 16384",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 22224011,
            "range": "± 212657",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4176988,
            "range": "± 66675",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 24031609,
            "range": "± 82786",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 224685230,
            "range": "± 765011",
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
          "id": "285b00cbe0346a1a1feb14158a0bc59484068e52",
          "message": "Remove tarpaulin from Devcontainer (#38)",
          "timestamp": "2022-11-14T15:53:50+01:00",
          "tree_id": "55fa5c5861051c753c7bcd31735bb7b90e39d017",
          "url": "https://github.com/eclipse/chariott/commit/285b00cbe0346a1a1feb14158a0bc59484068e52"
        },
        "date": 1668438943922,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 482865,
            "range": "± 9243",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2709785,
            "range": "± 10494",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25295762,
            "range": "± 256343",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4425741,
            "range": "± 22066",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26535172,
            "range": "± 94505",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 254341656,
            "range": "± 634309",
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
          "id": "b0d2add377f1740f26c52bfb4e234ce415b91b81",
          "message": "Put proto types in own crate, unifying imports (#32)",
          "timestamp": "2022-11-15T12:59:10+01:00",
          "tree_id": "22be5a51e8c5bb48133bf33c85c0ecae2369af51",
          "url": "https://github.com/eclipse/chariott/commit/b0d2add377f1740f26c52bfb4e234ce415b91b81"
        },
        "date": 1668514334097,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 434269,
            "range": "± 24298",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3043235,
            "range": "± 111060",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29299020,
            "range": "± 1252050",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 4173286,
            "range": "± 170691",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 28711743,
            "range": "± 1114679",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 281080313,
            "range": "± 8184612",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dariuszparys@users.noreply.github.com",
            "name": "Dariusz Parys",
            "username": "dariuszparys"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b850c220bbfe66c00e6d248101d1d965a25f83b2",
          "message": "Update rust-toolchain to TOML format (#43)",
          "timestamp": "2022-11-15T14:49:33+01:00",
          "tree_id": "da7174c7da3d25dc0b9f32bae469d3e18bd2ec4f",
          "url": "https://github.com/eclipse/chariott/commit/b850c220bbfe66c00e6d248101d1d965a25f83b2"
        },
        "date": 1668520745224,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 397123,
            "range": "± 12130",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2633384,
            "range": "± 10058",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25371143,
            "range": "± 30956",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3730547,
            "range": "± 29514",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26003484,
            "range": "± 96463",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 262975603,
            "range": "± 683033",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dariuszparys@users.noreply.github.com",
            "name": "Dariusz Parys",
            "username": "dariuszparys"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "95912b1ed06b67614bb952d8c678838ac6f7dc58",
          "message": "Fix Rust CI workflow (#44)",
          "timestamp": "2022-11-15T17:27:50+01:00",
          "tree_id": "06aefd6591b35b4b7bb11b605e81f784fecefd70",
          "url": "https://github.com/eclipse/chariott/commit/95912b1ed06b67614bb952d8c678838ac6f7dc58"
        },
        "date": 1668530208374,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 396269,
            "range": "± 5576",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2630223,
            "range": "± 4073",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25288227,
            "range": "± 25439",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3471436,
            "range": "± 77128",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 25912276,
            "range": "± 55037",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 252514694,
            "range": "± 440465",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dariuszparys@users.noreply.github.com",
            "name": "Dariusz Parys",
            "username": "dariuszparys"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8e2c46ff7c87ec2156d97b6c2317f89c865a8e1a",
          "message": "Remove Alex recommends GH workflow (#48)",
          "timestamp": "2022-11-17T10:07:56+01:00",
          "tree_id": "42a96aabd7fdbf9c309d6597f0258c2c230dc447",
          "url": "https://github.com/eclipse/chariott/commit/8e2c46ff7c87ec2156d97b6c2317f89c865a8e1a"
        },
        "date": 1668676415034,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 533742,
            "range": "± 33929",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3166897,
            "range": "± 142947",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29662993,
            "range": "± 495570",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 5216909,
            "range": "± 131224",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 31297483,
            "range": "± 491113",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 295849615,
            "range": "± 3188193",
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
          "id": "90dee0ea22f065f47fe9192f4e21c5667bbf061b",
          "message": "Update to .NET 7 SDK (#49)",
          "timestamp": "2022-11-17T16:59:22+01:00",
          "tree_id": "f057b3fe6a71a1b801d056cdf2b4ba1435c10d09",
          "url": "https://github.com/eclipse/chariott/commit/90dee0ea22f065f47fe9192f4e21c5667bbf061b"
        },
        "date": 1668703631820,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 526352,
            "range": "± 82364",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3197813,
            "range": "± 20448",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29967095,
            "range": "± 379238",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 5213081,
            "range": "± 136924",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 31607276,
            "range": "± 388491",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 301667595,
            "range": "± 8073104",
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
          "id": "76745e817d1335ab09342c30567e97da94724298",
          "message": "Fix CI workflow to install .NET 6 (#54)",
          "timestamp": "2022-11-18T09:44:40+01:00",
          "tree_id": "04d6bf3cf139fc7f4a991ccc62ab90bfeec2b396",
          "url": "https://github.com/eclipse/chariott/commit/76745e817d1335ab09342c30567e97da94724298"
        },
        "date": 1668761368674,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 397409,
            "range": "± 6091",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2625093,
            "range": "± 9849",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25289484,
            "range": "± 130079",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3624506,
            "range": "± 65381",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 25929333,
            "range": "± 66226",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 251127708,
            "range": "± 212237",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dariuszparys@users.noreply.github.com",
            "name": "Dariusz Parys",
            "username": "dariuszparys"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6d2e9db2303b3d8b88ed62ec0aa8ce92026c160e",
          "message": "Docs/update readme for podman (#45)\n\n* docs: update dev container usage\r\n\r\n- Clarifying the usage of dev containers in the current release\r\nlimited to WSL2 and Linux systems\r\n- Add section on how to use podman with dev containers in Chariott\r\n\r\n* docs: update block quote\r\n\r\n* docs: update readme\r\n\r\nAdded important note section to clarify the current support of\r\nWSL2 / linux with Ubuntu 20.04 on AMD64 architectures.\r\n\r\n* Update README.md\r\n\r\nCo-authored-by: Atif Aziz <code@raboof.com>\r\n\r\n* Apply suggestions from code review\r\n\r\nCo-authored-by: Atif Aziz <code@raboof.com>\r\n\r\n* Update README.md\r\n\r\nCo-authored-by: Atif Aziz <code@raboof.com>\r\n\r\n* docs: integrate PR feedback\r\n\r\nfixed also markdownlint issues\r\n\r\n* docs: moved requirements section\r\n\r\nCo-authored-by: Atif Aziz <code@raboof.com>",
          "timestamp": "2022-11-21T13:16:36+01:00",
          "tree_id": "66ccc9d1c674634822b8d18d0a5d54d194f6e4c9",
          "url": "https://github.com/eclipse/chariott/commit/6d2e9db2303b3d8b88ed62ec0aa8ce92026c160e"
        },
        "date": 1669034271522,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 398674,
            "range": "± 15141",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2684971,
            "range": "± 10907",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 25377059,
            "range": "± 43152",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3850805,
            "range": "± 31368",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26153728,
            "range": "± 71322",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 253016153,
            "range": "± 1130346",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "41898282+github-actions[bot]@users.noreply.github.com",
            "name": "github-actions[bot]",
            "username": "github-actions[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "20584e9935cad5c7d7e8bc90d86a5fdf909ed9b9",
          "message": "New notice file (#57)",
          "timestamp": "2022-11-24T13:04:31+01:00",
          "tree_id": "b054f7084fca73a680e00bc9ff788a06e1251092",
          "url": "https://github.com/eclipse/chariott/commit/20584e9935cad5c7d7e8bc90d86a5fdf909ed9b9"
        },
        "date": 1669292240094,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 371948,
            "range": "± 12107",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2705410,
            "range": "± 24028",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 26073397,
            "range": "± 229948",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3571321,
            "range": "± 88232",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26044646,
            "range": "± 67942",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 253113644,
            "range": "± 1632500",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "9027586+fprezado@users.noreply.github.com",
            "name": "fprezado",
            "username": "fprezado"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "cbe9e242ecc60c9536336171b6bc0ba9e989e20e",
          "message": "Merge pull request #87 from stepknees/steftom/contribupdate\n\nupdate contribution file",
          "timestamp": "2023-02-13T22:16:13Z",
          "tree_id": "ecb6c07f40a31765469c3d055c91a83bfc475e37",
          "url": "https://github.com/eclipse/chariott/commit/cbe9e242ecc60c9536336171b6bc0ba9e989e20e"
        },
        "date": 1676326839210,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 373137,
            "range": "± 9550",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2698781,
            "range": "± 15283",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 26123050,
            "range": "± 652186",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3698592,
            "range": "± 23627",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26764771,
            "range": "± 205499",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 255258264,
            "range": "± 578768",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "9027586+fprezado@users.noreply.github.com",
            "name": "fprezado",
            "username": "fprezado"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fdf90076c4c4cba71722f1d58fa30926cf8cc4ee",
          "message": "Merge pull request #93 from devkelley/devkelley/invoke_command_example\n\nAdded an invoke command example provider",
          "timestamp": "2023-03-07T19:33:08-08:00",
          "tree_id": "8381c32d5dea6f9a6a6bbd194b0c2196aa7035ad",
          "url": "https://github.com/eclipse/chariott/commit/fdf90076c4c4cba71722f1d58fa30926cf8cc4ee"
        },
        "date": 1678246660128,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 379640,
            "range": "± 7124",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 2675997,
            "range": "± 26551",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 26032318,
            "range": "± 309964",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 3708275,
            "range": "± 45936",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 26188657,
            "range": "± 44625",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 253066418,
            "range": "± 384988",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "9027586+fprezado@users.noreply.github.com",
            "name": "fprezado",
            "username": "fprezado"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bc98381c4a4cda398e4bca091e7f6c556833e3ab",
          "message": "Merge pull request #95 from ladatz/ladatz/headers\n\nUpdate License Banners according to Eclipse guidelines",
          "timestamp": "2023-03-10T09:54:10-08:00",
          "tree_id": "33acf006d5013d70428da762ef2cf1519991c651",
          "url": "https://github.com/eclipse/chariott/commit/bc98381c4a4cda398e4bca091e7f6c556833e3ab"
        },
        "date": 1678471194201,
        "tool": "cargo",
        "benches": [
          {
            "name": "ess/1-subscribers/1000-events",
            "value": 523770,
            "range": "± 49835",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/1000-events",
            "value": 3204025,
            "range": "± 45673",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/1000-events",
            "value": 29806685,
            "range": "± 792554",
            "unit": "ns/iter"
          },
          {
            "name": "ess/1-subscribers/10000-events",
            "value": 5344697,
            "range": "± 174210",
            "unit": "ns/iter"
          },
          {
            "name": "ess/10-subscribers/10000-events",
            "value": 31686290,
            "range": "± 440564",
            "unit": "ns/iter"
          },
          {
            "name": "ess/100-subscribers/10000-events",
            "value": 299976313,
            "range": "± 2667235",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}