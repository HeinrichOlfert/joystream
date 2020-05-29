import { newEntityId } from './EntityId.mock';
import { MusicMoodType } from '../schemas/music/MusicMood';

const values = [
  'Acerbic',
  'Aggressive',
  'Agreeable',
  'Airy',
  'Ambitious',
  'Amiable/Good-Natured',
  'Angry',
  'Angst-Ridden',
  'Anguished/Distraught',
  'Angular',
  'Animated',
  'Apocalyptic',
  'Arid',
  'Athletic',
  'Atmospheric',
  'Austere',
  'Autumnal',
  'Belligerent',
  'Benevolent',
  'Bitter',
  'Bittersweet',
  'Bleak',
  'Boisterous',
  'Bombastic',
  'Brash',
  'Brassy',
  'Bravado',
  'Bright',
  'Brittle',
  'Brooding',
  'Calm/Peaceful',
  'Campy',
  'Capricious',
  'Carefree',
  'Cartoonish',
  'Cathartic',
  'Celebratory',
  'Cerebral',
  'Cheerful',
  'Child-like',
  'Circular',
  'Clinical',
  'Cold',
  'Comic',
  'Complex',
  'Concise',
  'Confident',
  'Confrontational',
  'Cosmopolitan',
  'Crunchy',
  'Cynical/Sarcastic',
  'Dark',
  'Declamatory',
  'Defiant',
  'Delicate',
  'Demonic',
  'Desperate',
  'Detached',
  'Devotional',
  'Difficult',
  'Dignified/Noble',
  'Dramatic',
  'Dreamy',
  'Driving',
  'Druggy',
  'Earnest',
  'Earthy',
  'Ebullient',
  'Eccentric',
  'Ecstatic',
  'Eerie',
  'Effervescent',
  'Elaborate',
  'Elegant',
  'Elegiac',
  'Energetic',
  'Enigmatic',
  'Epic',
  'Erotic',
  'Ethereal',
  'Euphoric',
  'Exciting',
  'Exotic',
  'Explosive',
  'Extroverted',
  'Exuberant',
  'Fantastic/Fantasy-like',
  'Feral',
  'Feverish',
  'Fierce',
  'Fiery',
  'Flashy',
  'Flowing',
  'Fractured',
  'Freewheeling',
  'Fun',
  'Funereal',
  'Gentle',
  'Giddy',
  'Gleeful',
  'Gloomy',
  'Graceful',
  'Greasy',
  'Grim',
  'Gritty',
  'Gutsy',
  'Happy',
  'Harsh',
  'Hedonistic',
  'Heroic',
  'Hostile',
  'Humorous',
  'Hungry',
  'Hymn-like',
  'Hyper',
  'Hypnotic',
  'Improvisatory',
  'Indulgent',
  'Innocent',
  'Insular',
  'Intense',
  'Intimate',
  'Introspective',
  'Ironic',
  'Irreverent',
  'Jovial',
  'Joyous',
  'Kinetic',
  'Knotty',
  'Laid-Back/Mellow',
  'Languid',
  'Lazy',
  'Light',
  'Literate',
  'Lively',
  'Lonely',
  'Lush',
  'Lyrical',
  'Macabre',
  'Magical',
  'Majestic',
  'Malevolent',
  'Manic',
  'Marching',
  'Martial',
  'Meandering',
  'Mechanical',
  'Meditative',
  'Melancholy',
  'Menacing',
  'Messy',
  'Mighty',
  'Monastic',
  'Monumental',
  'Motoric',
  'Mysterious',
  'Mystical',
  'Naive',
  'Narcotic',
  'Narrative',
  'Negative',
  'Nervous/Jittery',
  'Nihilistic',
  'Nocturnal',
  'Nostalgic',
  'Ominous',
  'Optimistic',
  'Opulent',
  'Organic',
  'Ornate',
  'Outraged',
  'Outrageous',
  'Paranoid',
  'Passionate',
  'Pastoral',
  'Patriotic',
  'Perky',
  'Philosophical',
  'Plain',
  'Plaintive',
  'Playful',
  'Poignant',
  'Positive',
  'Powerful',
  'Precious',
  'Provocative',
  'Pulsing',
  'Pure',
  'Quirky',
  'Rambunctious',
  'Ramshackle',
  'Raucous',
  'Reassuring/Consoling',
  'Rebellious',
  'Reckless',
  'Refined',
  'Reflective',
  'Regretful',
  'Relaxed',
  'Reserved',
  'Resolute',
  'Restrained',
  'Reverent',
  'Rhapsodic',
  'Rollicking',
  'Romantic',
  'Rousing',
  'Rowdy',
  'Rustic',
  'Sacred',
  'Sad',
  'Sarcastic',
  'Sardonic',
  'Satirical',
  'Savage',
  'Scary',
  'Scattered',
  'Searching',
  'Self-Conscious',
  'Sensual',
  'Sentimental',
  'Serious',
  'Severe',
  'Sexual',
  'Sexy',
  'Shimmering',
  'Silly',
  'Sleazy',
  'Slick',
  'Smooth',
  'Snide',
  'Soft/Quiet',
  'Somber',
  'Soothing',
  'Sophisticated',
  'Spacey',
  'Sparkling',
  'Sparse',
  'Spicy',
  'Spiritual',
  'Spontaneous',
  'Spooky',
  'Sprawling',
  'Sprightly',
  'Springlike',
  'Stately',
  'Street-Smart',
  'Striding',
  'Strong',
  'Stylish',
  'Suffocating',
  'Sugary',
  'Summery',
  'Suspenseful',
  'Swaggering',
  'Sweet',
  'Swinging',
  'Technical',
  'Tender',
  'Tense/Anxious',
  'Theatrical',
  'Thoughtful',
  'Threatening',
  'Thrilling',
  'Thuggish',
  'Tragic',
  'Transparent/Translucent',
  'Trashy',
  'Trippy',
  'Triumphant',
  'Tuneful',
  'Turbulent',
  'Uncompromising',
  'Understated',
  'Unsettling',
  'Uplifting',
  'Urgent',
  'Virile',
  'Visceral',
  'Volatile',
  'Vulgar',
  'Warm',
  'Weary',
  'Whimsical',
  'Wintry',
  'Wistful',
  'Witty',
  'Wry',
  'Yearning' 
];

export const AllMusicMoods: MusicMoodType[] =
  values.map(value => ({ id: newEntityId(), value })) as unknown as MusicMoodType[] // A hack to fix TS compilation.

export const MusicMood = AllMusicMoods[0];
