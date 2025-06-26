module Orbits exposing (..)

type alias Orbit =
  { name : String
  , period : Float
  , energy : Float
  , bodies : List Body
  }

type alias Body =
  { frequencies : List FrequencyComponent
  }

type alias FrequencyComponent =
  { amplitude : Float
  , phase : Float
  , freq : Float
  }

orbits : List Orbit
orbits = [ { name = "Broucke A 2"
    , period = 7.702408
    , energy = -1.751113
    , bodies = [ { frequencies = [ { freq = -1
            , amplitude = 0.55295976
            , phase = 0
            }
          , { freq = -8, amplitude = 0.00144649, phase = -0 }
          , { freq = 6, amplitude = 0.21819415, phase = 3.14159265 }
          ]
        }
      , { frequencies = [ { freq = -1, amplitude = 0.55295978, phase = -0 }
          , { freq = -8, amplitude = 0.00144649, phase = -3.14159265 }
          , { freq = 6, amplitude = 0.21819415, phase = -0 }
          ]
        }
      , { frequencies = [ { freq = -1
            , amplitude = 1.10591955
            , phase = -3.14159265
            }
          ]
        }
      ]
    }
  , { name = "Figure 8"
    , period = 6.325897
    , energy = -1.287144
    , bodies = [ { frequencies = [ { freq = -1
            , amplitude = 0.5504203
            , phase = 2.37242761
            }
          , { freq = -2, amplitude = 0.16940572, phase = -1.29277482 }
          , { freq = -4, amplitude = 0.02798238, phase = 0.80165309 }
          , { freq = -5, amplitude = 0.01269615, phase = -2.86359165 }
          , { freq = -7, amplitude = 0.00293816, phase = -0.76910941 }
          , { freq = -8, amplitude = 0.00150222, phase = 1.84875632 }
          , { freq = 8, amplitude = 0.00150224, phase = 0.80169673 }
          , { freq = 7, amplitude = 0.00293816, phase = 0.27797878 }
          , { freq = 5, amplitude = 0.01269626, phase = 2.3724557 }
          , { freq = 4, amplitude = 0.02798231, phase = 1.84880704 }
          , { freq = 2, amplitude = 0.16940601, phase = -2.33995013 }
          , { freq = 1, amplitude = 0.55041951, phase = -2.86355993 }
          ]
        }
      , { frequencies = [ { freq = -1
            , amplitude = 0.55041951
            , phase = 0.27803272
            }
          , { freq = -2, amplitude = 0.16940601, phase = 0.80164252 }
          , { freq = -4, amplitude = 0.02798231, phase = -1.29278561 }
          , { freq = -5, amplitude = 0.01269626, phase = -0.76913695 }
          , { freq = -7, amplitude = 0.00293816, phase = -2.86361387 }
          , { freq = -8, amplitude = 0.00150224, phase = -2.33989592 }
          , { freq = 8, amplitude = 0.00150222, phase = -1.29283633 }
          , { freq = 7, amplitude = 0.00293816, phase = 2.37248325 }
          , { freq = 5, amplitude = 0.01269615, phase = 0.27800101 }
          , { freq = 4, amplitude = 0.02798238, phase = -2.33993957 }
          , { freq = 2, amplitude = 0.16940572, phase = 1.84881783 }
          , { freq = 1, amplitude = 0.5504203, phase = -0.76916504 }
          ]
        }
      , { frequencies = [ { freq = -1
            , amplitude = 0.55042001
            , phase = -1.81636124
            }
          , { freq = -2, amplitude = 0.1694026, phase = 2.89602802 }
          , { freq = -4, amplitude = 0.02798129, phase = 2.89602833 }
          , { freq = -5, amplitude = 0.01269555, phase = 1.32523562 }
          , { freq = -7, amplitude = 0.00293788, phase = 1.32523171 }
          , { freq = -8, amplitude = 0.00150205, phase = -0.24555688 }
          , { freq = 8, amplitude = 0.00150205, phase = 2.89603577 }
          , { freq = 7, amplitude = 0.00293788, phase = -1.81636095 }
          , { freq = 5, amplitude = 0.01269555, phase = -1.81635703 }
          , { freq = 4, amplitude = 0.02798129, phase = -0.24556433 }
          , { freq = 2, amplitude = 0.1694026, phase = -0.24556464 }
          , { freq = 1, amplitude = 0.55042001, phase = 1.32523141 }
          ]
        }
      ]
    }
  , { name = "Broucke A 15"
    , period = 92.056119
    , energy = -0.383678
    , bodies = [ { frequencies = [ { freq = -4
            , amplitude = 1.17234722
            , phase = 0
            }
          , { freq = -9, amplitude = 0.22526333, phase = 0 }
          , { freq = -14, amplitude = 0.07702826, phase = 0 }
          , { freq = -19, amplitude = 0.03217731, phase = -0 }
          , { freq = -24, amplitude = 0.01510481, phase = 0 }
          , { freq = -29, amplitude = 0.00755102, phase = 0 }
          , { freq = -34, amplitude = 0.00396727, phase = 0 }
          , { freq = -39, amplitude = 0.00214881, phase = 0 }
          , { freq = -44, amplitude = 0.00119634, phase = -0 }
          , { freq = 16, amplitude = 0.0023708, phase = -0 }
          , { freq = 11, amplitude = 0.00214533, phase = -0 }
          , { freq = 6, amplitude = 0.04255202, phase = 0 }
          , { freq = 1, amplitude = 2.77555386, phase = 3.14159265 }
          ]
        }
      , { frequencies = [ { freq = -4, amplitude = 1.17230886, phase = 0 }
          , { freq = -9, amplitude = 0.22526827, phase = -3.14159265 }
          , { freq = -14, amplitude = 0.07702932, phase = -0 }
          , { freq = -19, amplitude = 0.03217946, phase = 3.14159265 }
          , { freq = -24, amplitude = 0.01510537, phase = 0 }
          , { freq = -29, amplitude = 0.00755187, phase = 3.14159265 }
          , { freq = -34, amplitude = 0.00396744, phase = -0 }
          , { freq = -39, amplitude = 0.00214916, phase = -3.14159265 }
          , { freq = -44, amplitude = 0.00119634, phase = 0 }
          , { freq = 16, amplitude = 0.00236971, phase = 0 }
          , { freq = 11, amplitude = 0.00214524, phase = 3.14159265 }
          , { freq = 6, amplitude = 0.04255636, phase = -0 }
          , { freq = 1, amplitude = 2.77556554, phase = -0 }
          ]
        }
      , { frequencies = [ { freq = -4
            , amplitude = 2.34465608
            , phase = -3.14159265
            }
          , { freq = -14, amplitude = 0.15405759, phase = -3.14159265 }
          , { freq = -24, amplitude = 0.03021019, phase = -3.14159265 }
          , { freq = -34, amplitude = 0.00793471, phase = -3.14159265 }
          , { freq = -44, amplitude = 0.00239268, phase = -3.14159265 }
          , { freq = 16, amplitude = 0.00474051, phase = 3.14159265 }
          , { freq = 6, amplitude = 0.08510838, phase = 3.14159265 }
          ]
        }
      ]
    }
  ]