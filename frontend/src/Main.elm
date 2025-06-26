module Main exposing (main)

import Browser
import Browser.Events exposing (onAnimationFrameDelta)
import Canvas exposing (..)
import Canvas.Settings exposing (fill)
import Canvas.Settings.Advanced exposing (..)
import Color
import Html exposing (Html, div)
import Html.Attributes exposing (style)
import Orbits exposing (..)
import Html.Events
import Canvas.Settings.Line exposing (lineWidth)

width = 800
height = 800
centerX = width / 2
centerY = height / 2

type alias Model =
  { time : Float
  , orbit : Orbit
  }

type Msg
  = Frame Float
  | SetOrbit Orbit

defaultOrbit : Orbit
defaultOrbit =
  case List.head orbits of
    Just orbit -> orbit
    Nothing ->
      { name = "Invalid Orbit (no baked orbits found)"
      , period = 0.0
      , energy = 0.0
      , bodies = []
      }

main : Program () Model Msg
main =
  Browser.element
    { init = \() -> ( { time = 0, orbit = defaultOrbit }, Cmd.none )
    , view = view
    , update = update
    , subscriptions = \_ -> onAnimationFrameDelta Frame
    }

update : Msg -> Model -> ( Model, Cmd msg )
update msg model =
  case msg of
    Frame ms ->
      ( { model | time = model.time + ms / 1000.0 }, Cmd.none )
    SetOrbit orbit ->
      ( { model | orbit = orbit }, Cmd.none )


view : Model -> Html Msg
view { orbit, time } =
    div
        [ style "display" "flex"
        , style "justify-content" "center"
        , style "align-items" "center"
        , style "flex-direction" "column"
        ]
        [ Html.text orbit.name
        , Canvas.toHtml
            ( width, height )
            [ style "border" "10px solid rgba(0,0,0,0.1)" ]
            [ clearScreen
            , render orbit time
            ]
        , div [] (List.map orbitButton orbits)
        ]

orbitButton : Orbit -> Html Msg
orbitButton orbit =
  Html.button
    [ Html.Events.onClick (SetOrbit orbit) ]
    [ Html.text orbit.name ]

clearScreen : Canvas.Renderable
clearScreen = clear ( 0, 0 ) width height

render orbit time =
  let
    speed = 10.0 / orbit.period / (orbit.energy * orbit.energy)
    adjustedTime = speed * time
    bodies = renderBodies orbit speed adjustedTime
    trails = renderTrails orbit speed adjustedTime
    size = 100
  in
    group
      [ transform [ translate centerX centerY, scale size size ] ]
      [trails, bodies]

renderTrails : Orbit -> Float -> Float -> Canvas.Renderable
renderTrails orbit speed time =
  shapes
    [ Canvas.Settings.stroke Color.lightGray, lineWidth 0.04 ]
    (List.filterMap (renderTrail speed time) orbit.bodies)

renderTrail : Float -> Float -> Body -> Maybe Canvas.Shape
renderTrail speed time body = 
  let
    num = 40
    tail = 0.2 * speed
    substep = tail / num
    map = \offset -> (sampleBody (time - substep * toFloat offset) body)
    offsets = (List.range 0 num)
    samples = List.map map offsets
  in
    case samples of
      start :: rest -> Just (Canvas.path start (List.map Canvas.lineTo rest))
      _ -> Nothing

renderBodies : Orbit -> Float -> Float -> Canvas.Renderable
renderBodies orbit speed time =
  let
    num = 8
    tail = 0.02 * speed
    substep = tail / num
    subAlpha = 1.0 / num
    offsets = (List.range 0 num)
    map = \offset -> (renderBody orbit (time - substep * toFloat offset) subAlpha)
    shapes = List.map map offsets
    shadowConfig =
      { offset = (0, 0)
      , color = (Color.hsla 0.0 0.0 0.0 subAlpha)
      , blur = 8
      }
  in
    group
      [] -- [ shadow shadowConfig ]
      shapes

renderBody : Orbit -> Float -> Float -> Canvas.Renderable
renderBody orbit time subAlpha =
  let
    bodies = (sample time orbit)
    circles = List.map renderCircle bodies
  in
    shapes
      [ fill (Color.hsl 0.0 0.0 subAlpha)
      , compositeOperationMode Lighter
      ]
      circles

renderCircle : (Float, Float) -> Canvas.Shape
renderCircle coords = circle coords 0.2

sample : Float -> Orbit -> List (Float, Float)
sample at orbit = List.map (sampleBody at) orbit.bodies

sampleBody : Float -> Body -> (Float, Float)
sampleBody at body =
  let
    addCoord = \(x1, y1) -> (\(x2, y2) -> (x1 + x2, y1 + y2))
    components = (List.map (sampleFrequencyComponent at) body.frequencies)
  in List.foldl addCoord (0, 0) components

sampleFrequencyComponent : Float -> FrequencyComponent -> (Float, Float)
sampleFrequencyComponent at component =
  let
    theta = Basics.pi * 2 * at * component.freq + component.phase
    x = (Basics.cos -theta) * component.amplitude
    y = (Basics.sin -theta) * component.amplitude
  in (x, y)
