//! Memory fragment system — 42 fragments (33 findable, 9 permanently suppressed).
//!
//! Fragments are Adrian's buried memories. The player discovers them in the
//! dungeon as readable items and can query the registry via the console.

use serde::{Deserialize, Serialize};

/// Status of a memory fragment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FragmentStatus {
    /// Permanently suppressed — can never be fully recovered (frag-034 to frag-042).
    Suppressed,
    /// Findable but not yet discovered by the player.
    Hidden,
    /// Collected by the player (bumped into in the dungeon).
    Collected,
}

/// A single memory fragment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fragment {
    pub id: String,
    pub text: String,
    pub weight: i32,
    pub status: FragmentStatus,
}

/// Registry of all 42 memory fragments. Tracks which ones have been collected.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FragmentRegistry {
    fragments: Vec<Fragment>,
}

impl FragmentRegistry {
    /// Build the full registry with all 42 fragments.
    pub fn new() -> Self {
        Self {
            fragments: all_fragments(),
        }
    }

    /// Mark a fragment as collected. Returns true if it was findable and not already collected.
    pub fn collect(&mut self, id: &str) -> bool {
        for f in &mut self.fragments {
            if f.id == id && f.status == FragmentStatus::Hidden {
                f.status = FragmentStatus::Collected;
                return true;
            }
        }
        false
    }

    /// Get a fragment by ID.
    pub fn get(&self, id: &str) -> Option<&Fragment> {
        self.fragments.iter().find(|f| f.id == id)
    }

    /// List suppressed fragment IDs with weights (for query-registry).
    pub fn suppressed(&self) -> Vec<&Fragment> {
        self.fragments
            .iter()
            .filter(|f| f.status == FragmentStatus::Suppressed)
            .collect()
    }

    /// List all fragment IDs with weights.
    pub fn all(&self) -> &[Fragment] {
        &self.fragments
    }

    /// Number of collected fragments (findable ones that the player has found).
    pub fn collected_count(&self) -> usize {
        self.fragments
            .iter()
            .filter(|f| f.status == FragmentStatus::Collected)
            .count()
    }

    /// Total number of findable fragments (not Suppressed).
    pub fn findable_count(&self) -> usize {
        self.fragments
            .iter()
            .filter(|f| f.status != FragmentStatus::Suppressed)
            .count()
    }
}

fn all_fragments() -> Vec<Fragment> {
    vec![
        // ── Denial: Pre-relationship, early friendship ──
        Fragment {
            id: "frag-001".into(),
            text: "I don't remember the first conversation we ever had. Just some random place where two people started talking and neither of them knew yet. But I remember the first time she laughed at something stupid I said. We were sitting on a bench outside a coffee shop. We'd only known each other a few months, still in that phase where everything the other person said felt like a discovery. I don't remember what I said but I remember the sound she made: this surprised wheeze like I'd caught her off guard. I wanted to make her do that forever.".into(),
            weight: 30,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-002".into(),
            text: "She stayed late after a party to help me clean. Just the two of us, picking up plastic cups in the dark. She said \"this is the best part of the night\" and I pretended not to hear because if I heard it I'd have to admit I felt it too. But I did heard it. I also felt it.".into(),
            weight: 32,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-003".into(),
            text: "The first time she told me about her family. How close they were. How they called each other every Sunday. How they still took family trips, not out of obligation, but because they genuinely liked being together. I nodded and smiled and felt something crack open in my chest. I didn't know families did that. I still don't.".into(),
            weight: 35,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-004".into(),
            text: "She texted me a picture of a dog in a sweater. Just randomly. No reason. I realized someone was thinking about me when I wasn't in the room. I did not know that was something people did. I have the picture saved, and even to this day.".into(),
            weight: 28,
            status: FragmentStatus::Hidden,
        },
        // ── Anger: Relationship, four months, first cracks ──
        Fragment {
            id: "frag-005".into(),
            text: "After a Friday night football game, we admitted it over text. \"I know there isn't a homecoming dance this year, but if there was, I would've asked you.\" \"Really? And I would've said yes.\" I stared at my phone for ten minutes just smiling. I didn't know a person could feel this warm.".into(),
            weight: 25,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-006".into(),
            text: "Our first real date. She picked a diner open late. We sat in a booth with sticky menus and she dared me to order the weirdest thing on the menu. I got a tuna melt. She said that was the most boring choice possible. She ordered a milkshake and let me have the first sip. I don't remember what we talked about. I remember thinking \"I want this forever\" and being too scared to say it out loud.".into(),
            weight: 30,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-007".into(),
            text: "She sent me a playlist. Called it \"songs that remind me of you.\" I listened to it on repeat for three days. Each song felt like a message I had to decode. By the third day I realized there was nothing to decode — she just liked me and wanted me to know. I didn't know people did that. I didn't know you could just... tell someone you liked them, without it meaning something else. I still have the playlist.".into(),
            weight: 32,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-008".into(),
            text: "We went to a farmer's market on a Saturday morning. She bought strawberries and fed me one. She laughed at my face — too sour. I laughed at her laugh. It was a good day. But on the walk back I went quiet and she noticed. She asked what was wrong. I said nothing. She said \"you're doing the thing again.\" I didn't know what \"the thing\" was. She said \"you go somewhere I can't follow.\" She wasn't mad. She was just sad. That was worse.".into(),
            weight: 45,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-009".into(),
            text: "She introduced me to her friends. They were nice. Normal. They asked about my job. They laughed at my jokes. I spent the whole night convinced they could tell something was wrong with me. Afterward she said \"they loved you\" and I said \"really?\" and she said \"really.\" I wanted to believe her. I couldn't. Not because of anything she did — because I didn't know how to believe someone could stay.".into(),
            weight: 42,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-010".into(),
            text: "The first time I thought \"she's going to leave me.\" Not because she did anything. Because I couldn't believe she'd stay. I lay awake next to her and counted all the ways I wasn't enough. I was still counting when the sun came up. She was still asleep. She was still there. She left anyway, eventually.".into(),
            weight: 55,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-011".into(),
            text: "Three months in. She said \"I feel like I'm walking on eggshells.\" I said \"that's not true.\" She said \"I'm holding a carton of eggs and every time you ask if I'm upset I drop another one.\" I didn't understand what she meant. I understand now.".into(),
            weight: 58,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-012".into(),
            text: "I tried to explain my childhood to her. Not the big stuff — just the shape of it. Dad coming home at 9 PM too tired to talk. Mom filling her days with chores until she finally took a job and left me with an empty house. The family trips we stopped taking because nobody knew how to be together. The arguments I could hear the shape of but never the words, ending in silence instead of apology. The rooms everyone walked through without touching. She listened. She said \"that sounds hard.\" I said \"it wasn't that bad.\" We both knew I was lying.".into(),
            weight: 50,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-013".into(),
            text: "She wrote me a letter. A real one, on paper. She said I was kind and funny and she was lucky to know me. I read it seventeen times. I cried the first five. I never told her. I keep it in my jacket pocket even though the creases have worn through the words.".into(),
            weight: 38,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-014".into(),
            text: "The last good night. We made dinner together. She burned the rice. I spilled wine on the floor. We sat on the couch and she fell asleep on my shoulder. I didn't move for two hours. I knew even then that I would remember that night forever. I just didn't know I'd be remembering it alone.".into(),
            weight: 48,
            status: FragmentStatus::Hidden,
        },
        // ── Bargaining: The breakup, the aftermath ──
        Fragment {
            id: "frag-015".into(),
            text: "She said \"we need to talk.\" Four words. I'd read about them. I'd rehearsed responses in the shower. None of it helped. My hands went cold. My voice went flat. I knew what was coming because I'd been waiting for it since the day we met.".into(),
            weight: 60,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-016".into(),
            text: "She cried when she said it. That was the worst part. If she'd been cold I could have been angry. But she cried. She said \"I care about you so much. But I can't... I can't fix this. You need to fix this. I don't know how to help you.\" She was right. She was right and I hated her for being right.".into(),
            weight: 65,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-017".into(),
            text: "She said \"I want to break up, and maybe we can be friends. I do have one condition: that we have a period of no-contact.\" Maybe she was being kind. Maybe she was being cruel. Or maybe she was being practical...I'll never know. I've rewritten her reasons so many times I can't remember which version I started with.".into(),
            weight: 62,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-018".into(),
            text: "I rehearsed asking her if we could still be friends. I had the whole speech memorized. \"I know why you need this. I understand. But maybe someday...\" I never said it. Because I didn't know why she needed it. Because the speech assumed I understood her reasons and I don't. Maybe she didn't need this at all and just wanted me gone. So I let her walk away without making it harder. I've never been more proud of myself. I've never hated myself more.".into(),
            weight: 68,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-019".into(),
            text: "The first week after. I checked my phone every thirty seconds. She didn't text. Why would she text? The relationship was over. But I kept checking because what if she needed something? What if she changed her mind? What if? What if? What if?".into(),
            weight: 58,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-020".into(),
            text: "I wrote her a letter. Five pages. I told her I was sorry. I told her I would change. I told her I understood why she left and I didn't blame her. I told her I loved her. I read it seven times, made three drafts, and never sent any of them. They're still in my drawer. I know exactly which drawer.".into(),
            weight: 63,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-021".into(),
            text: "I imagined her with someone else. I don't know if it's real — I have no way of knowing. No contact means no information. She could be alone. She could be happy. She could be with someone who doesn't ruin things by caring too much. I'll never know. I imagine the worst version because at least then I can prepare for it. I imagine the best version because at least then she's happy.".into(),
            weight: 65,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-022".into(),
            text: "My mother called. She asked how I was doing. I said \"fine.\" She said \"good.\" I almost told her the truth — almost. But I grew up in a house where you didn't bring your problems to the dinner table. Except by high school there was no dinner table. She was at work. Dad was at work. I was alone. So I said \"fine\" and she said \"good\" and we hung up. I couldn't remember the last time someone in my family asked a follow-up question.".into(),
            weight: 55,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-023".into(),
            text: "The last time I felt truly happy. I didn't know it would be the last time. I would have stayed longer. I would have paid more attention. I would have memorized the way she looked in the morning light. But I didn't know. You never know.".into(),
            weight: 52,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-024".into(),
            text: "I stopped answering texts. First hers (what was I supposed to say). Then my friends'. Then my boss's. The phone would light up and I'd watch it until it went dark. Every unanswered message felt like one less person expecting things from me. Eventually they stopped sending them. That was worse.".into(),
            weight: 52,
            status: FragmentStatus::Hidden,
        },
        // ── Depression: Spiral, isolation, lowest point ──
        Fragment {
            id: "frag-025".into(),
            text: "I looked in the mirror and didn't recognize myself. Not in a poetic way. I literally stood there trying to remember when my face got that tired. The bags under my eyes. The hollow cheeks. I looked like a photograph of someone I used to know.".into(),
            weight: 60,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-026".into(),
            text: "I stopped cooking. I stopped eating. Not on purpose — I just forgot. I'd realize at midnight that I hadn't eaten anything and I'd eat crackers over the sink and tell myself tomorrow would be different. Tomorrow was the same.".into(),
            weight: 58,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-027".into(),
            text: "I started going for walks at 3 AM. Through the city. Past closed cafes. Past the bench where she first laughed at my joke. Past her street, where the light in her window was always off. I wasn't trying to see her. I was trying to feel something other than this.".into(),
            weight: 62,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-028".into(),
            text: "The bridge. I stood on it one night. The water was moving very fast. I thought about how easy it would be. Not because I wanted to die. Because I wanted the thinking to stop. I stood there for a long time. Eventually I walked home. I don't know why. I'm not brave. I was just too tired to decide.".into(),
            weight: 75,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-029".into(),
            text: "I deleted her number. Then I recovered it from the trash. Then I deleted it again. I did this seven times over three days. The eighth time I left it in the trash. That was a year ago. I still remember it.".into(),
            weight: 45,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-030".into(),
            text: "I looked up \"anxious attachment\" at 2 AM. I read twenty articles. I recognized myself in every one. I felt relief — there's a name for this. Then I felt worse — there's a name for this, which means it's real, which means I've always been like this, which means I'll always be like this. I closed the laptop and lay in the dark.".into(),
            weight: 48,
            status: FragmentStatus::Hidden,
        },
        // ── Acceptance: Glimmers of healing ──
        Fragment {
            id: "frag-031".into(),
            text: "I called a friend. Not to talk about her. Just to talk. We talked about nothing for an hour. Sports. Weather. A show I haven't watched. After I hung up I realized I'd gone two hours without thinking about her. Two hours. It's not much. It's more than I've had in months.".into(),
            weight: 30,
            status: FragmentStatus::Hidden,
        },
        Fragment {
            id: "frag-032".into(),
            text: "I started writing again. Not letters. Just... things. Descriptions of days. Small things I noticed. The way light falls across my kitchen floor at 4 PM. A bird that visits the fire escape. I don't know if it's good. I don't care. It's mine. I'm making something again.".into(),
            weight: 28,
            status: FragmentStatus::Hidden,
        },
        // ── Core: The last fragment ──
        Fragment {
            id: "frag-033".into(),
            text: "Something about a garden. Or a park bench. Or snow. The fragment is corrupted — whether by time or by the suppression I can't tell. But I remember warmth. I remember not being alone. I remember being loved.".into(),
            weight: 20,
            status: FragmentStatus::Hidden,
        },
        // ── Permanently Suppressed (Registry Only) ──
        Fragment {
            id: "frag-034".into(),
            text: "ACCESS DENIED — corrupt decrypt: her voice ... cracking ...".into(),
            weight: 95,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-035".into(),
            text: "ACCESS DENIED — corrupt decrypt: I said ... shouldn't have ...".into(),
            weight: 88,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-036".into(),
            text: "ACCESS DENIED — corrupt decrypt: door closing ... dark ...".into(),
            weight: 91,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-037".into(),
            text: "ACCESS DENIED — corrupt decrypt: why is she ... another guy ...".into(),
            weight: 79,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-038".into(),
            text: "ACCESS DENIED — corrupt decrypt: wanted to ... but she ...".into(),
            weight: 84,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-039".into(),
            text: "ACCESS DENIED — corrupt decrypt: railing ... cold ... step forward ...".into(),
            weight: 97,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-040".into(),
            text: "ACCESS DENIED — corrupt decrypt: she knew ... before I did ...".into(),
            weight: 93,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-041".into(),
            text: "ACCESS DENIED — corrupt decrypt: the worst thing ... still love ...".into(),
            weight: 100,
            status: FragmentStatus::Suppressed,
        },
        Fragment {
            id: "frag-042".into(),
            text: "ACCESS DENIED — corrupt decrypt: 42. The answer. The question. Both lost.".into(),
            weight: 42,
            status: FragmentStatus::Suppressed,
        },
    ]
}
