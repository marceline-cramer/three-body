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
