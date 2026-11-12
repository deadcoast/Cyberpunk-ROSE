"""Authoring script for data/barks.toml (Full Design v3.19, Narrative and Ivan: Bark pools). Run once; the TOML is the
source of truth after it. Each event has 8 untagged lines, 2 for each of Ivan's four Authority states and 2 for each of
the five strain classes; the Dissolution Rite has 4 of its own for D5 fed and for Runner dead; each chapter beat has
one. Lines are two sentences at most and name the world only through the string table's tokens ({order}, {city_above},
{fallen_corp}, {people}, {ferals}, {currency}, {city_below}); {name} is the slimo or law the event is about, {stratum}
the stratum reached.
"""
import os

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
CLASSES = ["garden", "hive", "runner", "bramble", "world_eater"]

# event: (any[8], states{1..4: [2]}, classes{name: [2]})
POOLS = {
    "haul": (
        ["Rig's heavy and the Runner's breathing. That's a good night down here.",
         "Spread it out on the floor. Let's see what the dark coughed up.",
         "Scrap in, Runner home. Write it in the book.",
         "Half of that's junk and the other half is junk we can sell.",
         "Every haul's a vote. Sell it, feed it, keep it: choose.",
         "Something in there rattles like money.",
         "Out of the dark and into the Stash. My favourite trip.",
         "Wet, filthy, unidentified. Just how I like my presents."],
        {1: ["Not much, but it's ours. Nobody gave us a thing down here.", "Keep them coming and maybe we eat this week."],
         2: ["The district's watching that haul come in. Good. Let them see it.", "We're getting good at this. Don't let it make you careless."],
         3: ["A city runs on hauls like that. Mine runs better.", "Log it, sort it, and send the Runner to a bed."],
         4: ["The empire eats tonight. Again.", "There's {people} up and down the stack who'd kill for that haul. They work for me instead."]},
        {"garden": ["Anything that glows goes to the Pits. Let the Mat drink the light.", "Clean water and cold light: that's what we sell them later."],
         "hive": ["Chrome for the cords. The walls want feeding.", "Every scrap of metal is armour we haven't grown yet."],
         "runner": ["Fast in, fast out. That's the Runner strain talking.", "The cords are already reaching for the next one."],
         "bramble": ["Feed the thorns. The sewerats like a thick Mat.", "More for the Pens and more for the brambles."],
         "world_eater": ["The Mat's hungry. It's always hungry.", "Toss it in. It'll eat the crate too."]}),
    "rare": (
        ["Now that's a Rare. Somebody up top is missing it.",
         "Look at the finish on that. Somebody paid real {currency} for it once.",
         "A Rare. Don't sell it to the first stall that blinks.",
         "That's the good stuff. Pin it before somebody feeds it to a Pit.",
         "Identified and worth it. Unlike most of my relatives.",
         "Rare. The {order} would call that contraband, which means it's valuable.",
         "Somebody's clinic is short one of those.",
         "Hold it up to the light. Not too long; somebody might see it."],
        {1: ["A Rare, in this dump. Maybe we're not dying after all.", "Keep that one close. We can't afford to lose it."],
         2: ["The districts will hear about that one by dinner.", "That buys us a law, or a lot of friends."],
         3: ["Rares like that keep a city honest. Mostly.", "Put it in the Vault, not on the floor."],
         4: ["Another Rare for the empire. I'm starting to expect them.", "File it with the others. The crown jewels are getting crowded."]},
        {"garden": ["Feed it and the Mat'll glow for a week.", "Light like that sells better than it burns."],
         "hive": ["Plate the cords with that and they'll stop bullets.", "Rare chrome makes rare armour."],
         "runner": ["That'll stretch the tips another block.", "Faster Mat, faster Runners. Feed it."],
         "bramble": ["The thorns would love that.", "Sharp enough for the brambles."],
         "world_eater": ["The Mat could eat that and ask for seconds.", "Shame to feed it. Shame not to."]}),
    "blackline": (
        ["That's Moon-make. Hold it close and they'll find you by the smell of their own money.",
         "Blackline. The {order} count every piece, and now they're one short.",
         "Holy hardware. Hide it before it hides you.",
         "That's the kind of chrome that comes with a search warrant.",
         "Blackline in the Stash. Somewhere a priest is checking an inventory.",
         "Don't sell that topside. They'll smell it from orbit.",
         "Moon-steel. It hums when you hold it. That's not a good sign.",
         "Every Blackline we keep is a lamp that comes looking."],
        {1: ["We're not ready for what that brings. Bury it deep.", "Lovely. Now we have something worth killing us for."],
         2: ["Put it in a Vault and put a Guard on the Vault.", "That's leverage. Let's not waste it."],
         3: ["Let them come for it. We've got walls now.", "Their gear, our city. Fair trade."],
         4: ["Add it to the war chest. The Moon's paying for its own funeral.", "I've got a whole room of that now. They must be furious."]},
        {"garden": ["Even Moon-steel glows when the Mat gets into it.", "Feed it slow. Blackline makes the light jumpy."],
         "hive": ["The cords will wear their own armour. Fitting.", "Their plating on our walls. I love it."],
         "runner": ["Blackline in the Pit and the tips will run like rats.", "That'll carry us up a stratum."],
         "bramble": ["Thorns with Moon-steel in them. Let them try the brambles now.", "Sharpen the hedges with it."],
         "world_eater": ["The Mat eats their gear. Next it eats them.", "Feed it to the hunger. The hunger knows what to do."]}),
    "runner_down": (
        ["{name} is down! Somebody get to them.",
         "Runner down. Sixty seconds, then it's a body.",
         "{name}, stay with me. Crawl if you have to.",
         "Down in the dark. Get up, {name}. Get up.",
         "They hit {name}. Find the shooter or find the exit.",
         "That's a lot of blood for one Runner. Move.",
         "{name}'s on the floor. Followers, earn your rations.",
         "Talk to me, {name}. Say something rude."],
        {1: ["Don't you die on me, {name}. We can't spare you.", "Get up. There's nobody coming."],
         2: ["The district's got a bed and a Medic waiting. Get there.", "{name}, you've got people. Crawl to them."],
         3: ["Every Runner's a citizen. Bring {name} home.", "I'll trade every crate on that route for {name}."],
         4: ["The empire doesn't leave its Runners on the floor.", "{name}. Up. That's an order."]},
        {"garden": ["Follow the light home, {name}.", "Think of the glow at the Heart. Crawl toward it."],
         "hive": ["The cords can carry you if you reach them.", "Armour up, {name}. Too late for that? Crawl."],
         "runner": ["The Mat's close, {name}. Reach for it.", "Run, crawl, slither. Just move."],
         "bramble": ["Get behind the thorns, {name}.", "The brambles will cover you. Crawl."],
         "world_eater": ["Don't let the Mat find you first, {name}.", "Get up before something hungry does."]}),
    "runner_dead": (
        ["{name}'s in the east drain. I'll tell the bunk. You tell me what they were carrying.",
         "{name} didn't come home. Put the name on the wall.",
         "We lost {name}. The route remembers where.",
         "That's {name} done. The rig's still down there.",
         "Dead in the dark. That's how most of us go.",
         "{name}. Somebody sing something. Not you.",
         "I'll miss {name}. I'll miss the haul more, and I'll hate myself for it.",
         "Write it down: {name}, a Runner, never came back."],
        {1: ["We can't afford funerals. We can barely afford {name}.", "One less mouth. I hate that I said that."],
         2: ["The district will want a name for the wall. Give them {name}.", "{name} ran for us. Remember the route."],
         3: ["A city mourns its Runners. This one will.", "Bring {name}'s rig home if it costs us another run."],
         4: ["The empire built itself on names like {name}.", "Carve {name} deep. Emperors have long memories."]},
        {"garden": ["Put a light on the wall for {name}.", "The Mat will glow a little dimmer tonight."],
         "hive": ["{name} should have had thicker plates. We'll grow them.", "Steel doesn't help the dead. Grow it anyway."],
         "runner": ["Too fast, too far. That's the Runner strain's price.", "{name} ran ahead of the Mat. Never again."],
         "bramble": ["The thorns weren't there for {name}. Grow them further.", "Plant something sharp where {name} fell."],
         "world_eater": ["The Mat will find the body before we do.", "Hunger takes. Today it took {name}."]}),
    "newcomer": (
        ["Another one wakes. Give them a name and a bed.",
         "Fresh out of the flush. Welcome to the bottom.",
         "{name} just opened their eyes. Somebody explain the smell.",
         "New body, new name. {name}, you live here now.",
         "The cure got another one. Their loss, our labour.",
         "{name}'s awake. Chrome scars and all.",
         "Somebody up there threw away a perfectly good {people}.",
         "Welcome, {name}. Rule one: never drink the black water."],
        {1: ["One more mouth. One more pair of hands. Fine.", "{name}, we've got almost nothing. Now you've got almost nothing too."],
         2: ["The district's got room. Get {name} to a Pod.", "We're growing. {name} makes us bigger."],
         3: ["{name} joins a city, not a sump. Lucky.", "Every newcomer is a citizen the {order} failed to kill."],
         4: ["The empire grows by one. {name}, bow if you like.", "Another subject. I'll try to be a good king."]},
        {"garden": ["Show {name} the light first. It helps.", "{name}'ll want to see the Lume Farms. Everybody does."],
         "hive": ["Get {name} some plating. It's dangerous out there.", "Welcome to the Hive, {name}. The walls hum."],
         "runner": ["{name} looks fast. Good.", "Point {name} at the Route Board."],
         "bramble": ["Mind the thorns, {name}.", "{name}, meet the sewerats. They're friendly now. Mostly."],
         "world_eater": ["Don't let the Mat taste you, {name}.", "Feed {name} before the Mat gets ideas."]}),
    "d5_fed": (
        ["Dropping one of ours in the Pit. Write the name on the wall first.",
         "{name} goes back to the Mat. That's the only funeral we can afford.",
         "He'd have done the same to me. Doesn't mean I'll sleep.",
         "Into the Pit. The Mat remembers everything it eats.",
         "{name} feeds the Heart now. Don't look at me like that.",
         "Somebody hold the wake. I'll hold the grudge.",
         "Not a slimo anymore. Just nutrient. God, I hate this place.",
         "We'll grow a cord where {name} goes in. That's the best I've got."],
        {1: ["We're starving. That's the excuse. It's a bad one.", "Forgive us, {name}. Or don't."],
         2: ["The district won't like it. They'll like starving less.", "Write it in the book with the rest of the bad decisions."],
         3: ["A city that eats its dead. Lovely. It's still a city.", "The law says it's fine. The law is a liar."],
         4: ["Empires are built on worse. That's not a comfort.", "Even kings feed their own to the dark."]},
        {"garden": ["{name}'s light goes into the Mat.", "Something will glow where {name} goes."],
         "hive": ["Chrome in the bones. The cords will take it.", "{name}'s plating goes into the walls."],
         "runner": ["{name} will run with the tips now.", "Feed the reach. {name} would've wanted to travel."],
         "bramble": ["The thorns will have a new root.", "{name} feeds the brambles."],
         "world_eater": ["The hunger takes {name}. It always takes.", "Feed it. It's never full."]}),
    "patrol": (
        ["White light in the culverts. The Moon sent a cleaning crew.",
         "Two lamps, coming down. Douse the Bulbs.",
         "A Patrol. They map, they leave. Let them leave.",
         "Scouts at the grate. Don't shoot unless you mean it.",
         "The {order} are counting cords again.",
         "Lamps on the ladder. Everybody be boring for a minute.",
         "White light means somebody's writing us down.",
         "Here comes the census, robes and all."],
        {1: ["Hide everything. They'll see a dump and leave.", "Don't let them find the Heart. Not yet."],
         2: ["Let them map. Maps get old.", "They're looking for a city. Let's not show them one."],
         3: ["A Patrol. Cute. Let them count the turrets too.", "They can map what they like. It'll be different tomorrow."],
         4: ["Two scouts for an empire. They're insulting me.", "Let them map the edge of my kingdom. They'll never see the middle."]},
        {"garden": ["Kill the light. All of it. Now.", "Every Bulb's a beacon. Dim them."],
         "hive": ["Let them look at the walls. Let them worry.", "Scouts on our armour. Good. Let them report it."],
         "runner": ["The tips will be somewhere else by the time they draw it.", "Map that, you slow bastards."],
         "bramble": ["Walk into the thorns, I dare you.", "The brambles don't need light to bite."],
         "world_eater": ["Careful where you stand, scouts. The ground's hungry.", "Lamps make good snacks."]}),
    "sweep": (
        ["A Sweep. Bleach and lamps. Salt the cords.",
         "Six of them coming down. That's not a census, that's a cleaning.",
         "Sprayers. Keep the water off the cords.",
         "They're sweeping the brightest thing they can find. Don't be it.",
         "Here comes the bleach. Every Guard, up.",
         "A Sweep down the grate. They mean it this time.",
         "Get the slimos out of the light. Panic later.",
         "Bleach in the water. Somebody close a Gate."],
        {1: ["We can't fight that. Run, hide, regrow.", "Let them burn the edge. Save the Heart."],
         2: ["The districts can hold. Tell them I said so.", "Guards to the Gates. Make it cost them."],
         3: ["A Sweep against a city. Let's teach them arithmetic.", "Turrets, Gates, salt. We built this for today."],
         4: ["Six Oathkeepers against an empire. Send them home in a bucket.", "My city doesn't get swept. It sweeps back."]},
        {"garden": ["The light gave us away. Worth it. Dim it now.", "Save the Lungs. Lose the Bulbs if you have to."],
         "hive": ["Let the bleach hit armour. It'll bounce.", "The walls hold. That's what they're for."],
         "runner": ["Grow around them. They're slow.", "Cut and regrow. The Mat's faster than bleach."],
         "bramble": ["Thorns at every crossing. Let them spray those.", "The sewerats will greet them."],
         "world_eater": ["Six more mouths for the Mat. Let it eat.", "Bleach doesn't stop hunger."]}),
    "cleansing": (
        ["They're coming down with fire. This is a Cleansing.",
         "Lamps first, then the bleach, then the flamers. Know the order.",
         "The sluice is open. Get the cords out of the water.",
         "Flamers on the ladder. Every Gate, every Guard.",
         "This is what they came from the Moon for.",
         "White light, white robes, white fire. Hold the Heart.",
         "Here comes the cure, the whole dose.",
         "If you're going to pray, pray to the Gates."],
        {1: ["We're not ready. We were never going to be ready.", "Hold the Heart. Everything else regrows."],
         2: ["Districts, this is what you built your walls for.", "Make them pay for every tile."],
         3: ["A city doesn't burn easy. Prove it.", "Every turret, every Gate, every Guard: now."],
         4: ["They want to cleanse an empire. Let them try.", "Come down, then. I've waited years for this."]},
        {"garden": ["They'll burn the light first. Let them find the Lungs instead.", "Dim everything. Make them hunt in the dark."],
         "hive": ["Let the fire hit the armour. Let it hit it all night.", "Walls and wires. Let them melt their flamers on us."],
         "runner": ["Grow faster than they burn.", "Every tile they torch, grow two."],
         "bramble": ["Feed them to the thorns.", "Let the sewerats out. All of them."],
         "world_eater": ["Let the Mat taste a flamer. It's been curious.", "They brought fire to a hunger fight."]}),
    "bloom": (
        ["That one's gone sewerat. It still has your face in its head somewhere.",
         "A Bloom. Cut-off tissue gone wild.",
         "Something feral is eating the edge.",
         "Crawlers out of a Bloom. Guards to the Pens.",
         "The Mat out there isn't ours anymore.",
         "Feral strain. It'll eat the route home if we let it.",
         "A Bloom. Somewhere a Relay failed and nobody noticed.",
         "Lost tissue, lost minds. Watch the drains."],
        {1: ["We can't fight a Bloom and the {order}. Pick one.", "Sever it, or starve it, or run."],
         2: ["The districts on the edge won't sleep tonight.", "Get a Pen near it. Tame what comes."],
         3: ["A Bloom inside city limits. Unacceptable.", "Reclaim it or burn it. I don't care which."],
         4: ["Even the wild parts of my empire answer eventually.", "Bring the Bloom to heel. It's still ours underneath."]},
        {"garden": ["Shine the signal at it. Maybe it remembers.", "Light it up and take it back."],
         "hive": ["Wall it off. Let it eat the armour.", "The cords are walls. Make them one."],
         "runner": ["Reach it first and it's ours again.", "Outgrow it."],
         "bramble": ["A Bloom? That's just a bramble with no manners.", "The Pens are hungry. Bring me crawlers."],
         "world_eater": ["Two hungers in one drain. Ours is bigger.", "Let ours eat theirs."]}),
    "flood": (
        ["Rain up top. Down here that's a tide.",
         "The water's rising. Get the Pods off the floor.",
         "Flood shift. Lungs, do your job.",
         "Here comes the tide. Everything low gets wet.",
         "Rain on {city_above}, rivers in {city_below}.",
         "Black water's coming up. Mind the Deposits.",
         "The Sump drinks tonight.",
         "Water at the doors. Close the Gates on it."],
        {1: ["We'll lose a Pod. We always lose a Pod.", "Climb. Just climb."],
         2: ["The districts know the drill. Up and dry.", "Count the Lungs. Grow another."],
         3: ["A city with Lungs laughs at rain.", "Let it rise. We built high."],
         4: ["The tide comes in and the empire stays dry.", "Rain is for peasants."]},
        {"garden": ["The Lungs will take it. That's what they're for.", "Clean water later, if we live."],
         "hive": ["The walls hold water too.", "Gate it out."],
         "runner": ["The cords will reach the dry tiles.", "Grow up the walls. Out of the wet."],
         "bramble": ["The sewerats can swim. Can you?", "Wet thorns are still thorns."],
         "world_eater": ["The Mat drinks. It's fine.", "Hungry and wet. Lovely."]}),
    "law": (
        ["Signed. The wave's leaving the Heart.",
         "{name} is law now. The far districts will hear it soon.",
         "There. A law. Somebody frame it.",
         "The Charter grows. So does the paperwork.",
         "Signed in slime. It's binding.",
         "{name}. Let's see who complains first.",
         "A law down here is a promise up the cords.",
         "I signed it. Blame me."],
        {1: ["A law. We're practically a government.", "Somebody tell me how this works."],
         2: ["The districts voted. Mostly with their feet.", "{name} for the people. Some of them."],
         3: ["A city needs laws. Mine has good ones.", "Sign it, send it, enforce it."],
         4: ["By royal decree: {name}.", "An emperor's signature, still wet."]},
        {"garden": ["Laws grow like light down here.", "A clean law for a clean city."],
         "hive": ["The wires carry it faster.", "Hard-wired. Like the walls."],
         "runner": ["The signal will carry it before dinner.", "Fast cords, fast laws."],
         "bramble": ["Sharp law for a sharp city.", "Thorny, but fair."],
         "world_eater": ["Hungry law. Good.", "Law is just appetite with paperwork."]}),
    "stratum": (
        ["You hear that? That's traffic. We're under their feet.",
         "We're through. {stratum}, here we come.",
         "The Mat's up the shaft. New dark, new scrap.",
         "{stratum}. Different smell, same filth.",
         "One stratum closer to the street.",
         "Up through the slab. Nobody's ever done that before. Nobody sane.",
         "The first tip is in {stratum}. Follow it.",
         "New ground. New grates. New trouble."],
        {1: ["We're climbing out of the Sump. Finally.", "{stratum}. Higher than I ever thought we'd get."],
         2: ["The district will want a name for it.", "{stratum}, claimed. Get the Relays up."],
         3: ["The city climbs. The {order} should be worried.", "{stratum} is ours. Next."],
         4: ["Another floor of the empire.", "The stack is mine, one slab at a time."]},
        {"garden": ["Light up {stratum}. Let it glow.", "The Lume will reach the top yet."],
         "hive": ["Wire {stratum}. Armour every shaft.", "The walls climb with us."],
         "runner": ["The tips didn't even slow down.", "Up and up. That's the Runner strain."],
         "bramble": ["Thorns in {stratum}. Make it ours.", "Plant the brambles at the top of the shaft."],
         "world_eater": ["A whole new stratum to eat.", "The hunger climbs."]}),
}

RITE = {
    "d5_fed": ["The Rite says it's fine. My stomach disagrees.", "Another one for the Rite. Write the name, then drop.",
               "The Rite makes it routine. Routine is the worst part.", "By the Rite, {name} feeds the Heart."],
    "runner_dead": ["{name} goes to the Pit by the Rite. Nothing wasted, everything lost.",
                    "Under the Rite even our dead feed the city. Tell {name}'s bunk.",
                    "The Rite takes {name}. I'll take the grief.", "Dead and dissolved. That's the law I signed."],
}

BEATS = {
    1: "The Heart's beating and eight of us have beds. That's a city, near enough.",
    2: "First Forge, first sale. Now we're a market.",
    3: "We know what the {order} are now. Good. No more surprises.",
    4: "The Bloom's done and {name}'s on the wall.",
    5: "The war chest is real, and it's under our feet.",
    6: "We've got friends up top. Don't trust them, but keep them.",
    7: "That's the end of it. Look at what we became.",
}


def q(s):
    return '"' + s.replace('\\', '\\\\').replace('"', '\\"') + '"'


def main():
    out = ["# Cyberpunk: ROSE — Ivan's bark pools (Full Design v3.19, Narrative and Ivan: Bark pools). Written by",
           "# tools/gen/gen_barks.py once; this file is the source of truth. A line: event, and optionally state (1-4, Ivan's",
           "# Authority state), class (the strain's dominant class), rite (the Dissolution Rite signed), beat (a chapter).",
           ""]
    n = 0
    for ev, (anyl, states, classes) in POOLS.items():
        assert len(anyl) == 8 and all(len(v) == 2 for v in states.values()) and len(states) == 4
        assert sorted(classes) == sorted(CLASSES) and all(len(v) == 2 for v in classes.values())
        for t in anyl:
            out.append(f"[[bark]]\nevent = {q(ev)}\ntext = {q(t)}\n"); n += 1
        for s, ls in states.items():
            for t in ls:
                out.append(f"[[bark]]\nevent = {q(ev)}\nstate = {s}\ntext = {q(t)}\n"); n += 1
        for c, ls in classes.items():
            for t in ls:
                out.append(f"[[bark]]\nevent = {q(ev)}\nclass = {q(c)}\ntext = {q(t)}\n"); n += 1
    for ev, ls in RITE.items():
        for t in ls:
            out.append(f"[[bark]]\nevent = {q(ev)}\nrite = true\ntext = {q(t)}\n"); n += 1
    for ch, t in BEATS.items():
        out.append(f"[[bark]]\nevent = \"beat\"\nbeat = {ch}\ntext = {q(t)}\n"); n += 1
    with open(os.path.join(ROOT, "data", "barks.toml"), "w") as f:
        f.write("\n".join(out))
    print(f"wrote data/barks.toml: {n} lines")


if __name__ == "__main__":
    main()
