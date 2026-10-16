import React from 'react';
import {Composition} from 'remotion';
import {Film} from './Film';
import {Screen, SCREENS} from './Screens';
import {Clip, CLIPS, CLIP_FPS} from './Clips';
import TL from './timeline.json';

export const Root: React.FC = () => (
  <>
    <Composition id="PinPinPromo" component={Film} durationInFrames={Math.round(TL.duration * TL.fps)} fps={TL.fps} width={1920} height={1080} />
    {Object.entries(SCREENS).map(([id, aspect]) => (
      <Composition key={id} id={`screen-${id}`} component={Screen} defaultProps={{id}} durationInFrames={1} fps={30} width={1600} height={Math.round(1600 * aspect)} />
    ))}
    {Object.entries(CLIPS).map(([id, seconds]) => (
      <Composition key={id} id={`clip-${id}`} component={Clip} defaultProps={{id}} durationInFrames={Math.round(seconds * CLIP_FPS)} fps={CLIP_FPS} width={1920} height={1080} />
    ))}
  </>
);
