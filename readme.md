## Demo


## Comabat System

Attacks:
- Light
    - Normal
    - Forward
    - Up
    - Down
- Heavy
    - Normal
    - Forward
    - Up
    - Down
- Fake attack / Animation cancel
    - You can cancel animation within setup frames or change attack if second setup frame is inside window of the first
- Shield
    - Reduced damage
- Parry
    - Frame advantage


need to know precisesly how many frame per sprite


CharDev
- Create sketch animation sheet
- Create hitbox layout
- Transfer hitbox data into .ron
- Chain Buffer or Perfect Link
    - Linking windows

```ron
    Move(
        //what game object (character, prop)
        character_index: 0,
        input: "x",
        name: "",
        frames: [
            (
                 //what animation frame is this reffering to in the char sheet
                index: 1,
                //how this move/frame moves the subject relatively to previous frame
                position: (x: 0, y: 0),
                fps_duration: 3,
                fps_cooldown: 3,
                //hitboxes for this frame
                //position, lenght and height
                hitboxes: [
                    (x: 10, y: 12, length: 15, height: 15, damage: 1)
                ],
            )
            ..
        ],
        projectile: ()
    )

```

```ron
    Move(
        state: LightAttack,
        damage: 80,
        hitStunFrames: 2,
        //animation that last 5 frames, only 2 frames have hitbox, and it starts after the first one
        animation: (
            startIndex: 1,
            endIndex: 4,
            startupFrames: 1,
            activeFrames: 2,
            recoveryFrames: 2,
        ),
        hitbox: ( x: 10, y: 10, length: 10, height: 10 ),
        hurtbox: ( x: 10, y: 10, length: 10, height: 10 ),
        spawn: Some((
            direction: (x: 0, y: 1),
            speed: 2,
            lifetime: 5,
            hitbox: ()
        )),
        movement: Some((
            direction: (x: 0, y: 1),
            speed: 2,
        ))

    )
```

```ron
    Character(
        name: "",
        sprite_sheet: "",
        hurtbox: ( x: 10, y: 10, length: 10, height: 10 ),
        moveset: (
            light: Move(),
            heavy: Move()
        )
    )
```
