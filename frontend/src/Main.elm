module Main exposing (main)

import Browser
import Browser.Events exposing (onAnimationFrameDelta)
import Canvas exposing (..)
import Canvas.Settings exposing (fill)
import Canvas.Settings.Advanced exposing (..)
import Color
import Html exposing (Html, div)
import Html.Attributes exposing (style)

width = 400
height = 400
centerX = width / 2
centerY = height / 2

type alias Model =
  { time : Float
  }

type Msg = Frame Float


main : Program () Model Msg
main =
  Browser.element
    { init = \() -> ( { time = 0 }, Cmd.none )
    , view = view
    , update = update
    , subscriptions = \_ -> onAnimationFrameDelta Frame
    }

update : Msg -> Model -> ( Model, Cmd msg )
update msg model =
  case msg of
    Frame ms ->
      ( { model | time = model.time + ms / 1000.0 }, Cmd.none )


view : Model -> Html Msg
view { time } =
    div
        [ style "display" "flex"
        , style "justify-content" "center"
        , style "align-items" "center"
        ]
        [ Canvas.toHtml
            ( width, height )
            [ style "border" "10px solid rgba(0,0,0,0.1)" ]
            [ clearScreen
            , render time
            ]
        ]


clearScreen : Canvas.Renderable
clearScreen = clear ( 0, 0 ) width height

render time =
  let
    num = 10
    tail = 0.02
    substep = tail / num
    subAlpha = 1.0 / num
    offsets = (List.range 0 num)
    map = \offset -> (renderOne (time - substep * toFloat offset) subAlpha)
  in
    group [] (List.map map offsets)

renderOne time subAlpha =
  let
    size = width / 3
    rotation = degrees (time * 100)
    x = size * cos rotation
    y = size * sin rotation
    shadowConfig =
      { offset = (0, 0)
      , color = Color.black
      , blur = 10
      }
  in
    shapes
      [ transform [ translate centerX centerY ]
      , fill Color.white
      , alpha subAlpha
      , compositeOperationMode Screen
      , shadow shadowConfig
      ]
      [ circle ( x, y ) 10 ]
